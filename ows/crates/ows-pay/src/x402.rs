use base64::{engine::general_purpose::STANDARD as B64, Engine};

use crate::chains::{self, ChainMapping};
use crate::error::PayError;
use crate::types::{
    Eip3009Authorization, Eip3009Payload, PayResult, PaymentPayload, PaymentRequirements,
    X402Response,
};

/// Header some servers set on 402 responses (base64-encoded JSON).
const HEADER_PAYMENT_REQUIRED: &str = "x-payment-required";
/// Header we send back with the signed payment.
const HEADER_PAYMENT: &str = "X-PAYMENT";

/// Make an HTTP request, auto-handle 402 payment via x402, and return the result.
pub async fn pay(
    wallet_name: &str,
    passphrase: &str,
    url: &str,
    method: &str,
    body: Option<&str>,
) -> Result<PayResult, PayError> {
    let client = reqwest::Client::new();

    // Step 1: Make the initial request.
    let initial = build_request(&client, url, method, body, None)
        .send()
        .await?;

    // Not a 402 — return directly.
    if initial.status().as_u16() != 402 {
        let status = initial.status().as_u16();
        let text = initial.text().await.unwrap_or_default();
        return Ok(PayResult {
            status,
            body: text,
            amount_display: String::new(),
            network: String::new(),
        });
    }

    // Step 2: Parse 402 response to get payment requirements.
    // Some servers put it in the x-payment-required header (base64),
    // others put it in the response body as JSON.
    let requirements = parse_402_response(initial).await?;

    // Pick the first "exact" scheme on an EVM chain we support.
    let (req, chain) = pick_payment_option(&requirements)?;

    // Step 3: Get wallet's EVM address.
    let wallet_info = ows_lib::get_wallet(wallet_name, None)?;
    let evm_address = wallet_info
        .accounts
        .iter()
        .find(|a| a.chain_id.starts_with("eip155:"))
        .ok_or_else(|| PayError::Wallet("wallet has no EVM account".into()))?
        .address
        .clone();

    // Step 4: Build EIP-712 typed data for TransferWithAuthorization (EIP-3009).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let valid_after = now.saturating_sub(5);
    let valid_before = now + req.max_timeout_seconds;

    let mut nonce_bytes = [0u8; 32];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| PayError::Signing(format!("rng failed: {e}")))?;
    let nonce_hex = format!("0x{}", hex::encode(nonce_bytes));

    // Token name/version from extra metadata (needed for EIP-712 domain).
    let token_name = req
        .extra
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("USD Coin");
    let token_version = req
        .extra
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("2");

    // Chain ID number for EIP-712 domain.
    let chain_id_num: u64 = chain
        .caip2
        .split(':')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| PayError::Protocol(format!("bad CAIP-2: {}", chain.caip2)))?;

    let typed_data_json = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "version", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "TransferWithAuthorization": [
                { "name": "from", "type": "address" },
                { "name": "to", "type": "address" },
                { "name": "value", "type": "uint256" },
                { "name": "validAfter", "type": "uint256" },
                { "name": "validBefore", "type": "uint256" },
                { "name": "nonce", "type": "bytes32" }
            ]
        },
        "primaryType": "TransferWithAuthorization",
        "domain": {
            "name": token_name,
            "version": token_version,
            "chainId": chain_id_num.to_string(),
            "verifyingContract": req.asset
        },
        "message": {
            "from": evm_address,
            "to": req.pay_to,
            "value": req.amount,
            "validAfter": valid_after.to_string(),
            "validBefore": valid_before.to_string(),
            "nonce": nonce_hex.clone()
        }
    })
    .to_string();

    // Step 5: Sign with OWS.
    let sign_result = ows_lib::sign_typed_data(
        wallet_name,
        chain.ows_chain,
        &typed_data_json,
        Some(passphrase),
        None,
        None,
    )
    .map_err(|e| PayError::Signing(format!("EIP-712 signing failed: {e}")))?;

    let signature = format!("0x{}", sign_result.signature);

    // Step 6: Build PaymentPayload.
    let payload = PaymentPayload {
        x402_version: 1,
        scheme: "exact".into(),
        network: req.network.clone(),
        payload: Eip3009Payload {
            signature,
            authorization: Eip3009Authorization {
                from: evm_address,
                to: req.pay_to.clone(),
                value: req.amount.clone(),
                valid_after: valid_after.to_string(),
                valid_before: valid_before.to_string(),
                nonce: nonce_hex,
            },
        },
    };

    let payload_json = serde_json::to_string(&payload)?;
    let payload_b64 = B64.encode(payload_json.as_bytes());

    let amount_display = crate::discovery::format_usdc(&req.amount);

    // Step 7: Retry request with payment header.
    let retry = build_request(&client, url, method, body, Some(&payload_b64))
        .send()
        .await?;

    let status = retry.status().as_u16();
    let response_body = retry.text().await.unwrap_or_default();

    Ok(PayResult {
        status,
        body: response_body,
        amount_display,
        network: chain.name.to_string(),
    })
}

/// Parse a 402 response to extract PaymentRequirements.
///
/// Tries two approaches:
///   1. `x-payment-required` header (base64-encoded JSON)
///   2. Response body as JSON (how most real x402 servers work)
async fn parse_402_response(resp: reqwest::Response) -> Result<Vec<PaymentRequirements>, PayError> {
    // Try header first.
    if let Some(header_val) = resp.headers().get(HEADER_PAYMENT_REQUIRED) {
        if let Ok(header_str) = header_val.to_str() {
            if let Ok(decoded) = B64.decode(header_str) {
                if let Ok(parsed) = serde_json::from_slice::<X402Response>(&decoded) {
                    if !parsed.accepts.is_empty() {
                        return Ok(parsed.accepts);
                    }
                }
            }
        }
    }

    // Fallback: parse response body as JSON.
    let body_text = resp
        .text()
        .await
        .map_err(|e| PayError::Protocol(format!("failed to read 402 body: {e}")))?;

    let parsed: X402Response = serde_json::from_str(&body_text).map_err(|e| {
        PayError::Protocol(format!(
            "failed to parse 402 response as x402: {e}\nbody: {body_text}"
        ))
    })?;

    if parsed.accepts.is_empty() {
        return Err(PayError::Protocol("402 response has empty accepts".into()));
    }

    Ok(parsed.accepts)
}

/// Pick the best payment option from the requirements.
///
/// Handles both CAIP-2 identifiers ("eip155:8453") and plain names ("base").
fn pick_payment_option(
    requirements: &[PaymentRequirements],
) -> Result<(&PaymentRequirements, &'static ChainMapping), PayError> {
    for req in requirements {
        if req.scheme != "exact" {
            continue;
        }
        // Try CAIP-2 first, then plain chain name.
        if let Some(chain) =
            chains::chain_by_caip2(&req.network).or_else(|| chains::chain_by_name(&req.network))
        {
            return Ok((req, chain));
        }
    }

    let networks: Vec<_> = requirements.iter().map(|r| r.network.as_str()).collect();
    Err(PayError::Unsupported(format!(
        "no supported EVM chain in 402 response (networks: {networks:?})"
    )))
}

fn build_request(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    body: Option<&str>,
    payment_header: Option<&str>,
) -> reqwest::RequestBuilder {
    let mut req = match method.to_uppercase().as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    if let Some(b) = body {
        req = req
            .header("content-type", "application/json")
            .body(b.to_string());
    }

    if let Some(payment) = payment_header {
        req = req.header(HEADER_PAYMENT, payment);
    }

    req
}
