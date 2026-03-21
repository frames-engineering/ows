use crate::error::PayError;
use crate::types::{DiscoveredService, DiscoveryResponse, MppService, MppServicesResponse};

/// Coinbase CDP facilitator Bazaar discovery endpoint.
const CDP_DISCOVERY_URL: &str = "https://api.cdp.coinbase.com/platform/v2/x402/discovery/resources";

/// MPP services directory.
const MPP_SERVICES_URL: &str = "https://mpp.dev/api/services";

// ---------------------------------------------------------------------------
// x402 discovery
// ---------------------------------------------------------------------------

/// Fetch x402 services from the Bazaar.
pub async fn discover_x402(
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<DiscoveredService>, PayError> {
    let client = reqwest::Client::new();

    let resp = client
        .get(CDP_DISCOVERY_URL)
        .query(&[
            ("limit", limit.unwrap_or(100).to_string()),
            ("offset", offset.unwrap_or(0).to_string()),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(PayError::Http(format!(
            "x402 discovery returned {status}: {body}"
        )));
    }

    let body: DiscoveryResponse = resp
        .json()
        .await
        .map_err(|e| PayError::Protocol(format!("failed to parse x402 discovery response: {e}")))?;

    Ok(body.items)
}

// ---------------------------------------------------------------------------
// MPP discovery
// ---------------------------------------------------------------------------

/// Fetch all MPP services from mpp.dev/api/services.
pub async fn discover_mpp() -> Result<Vec<MppService>, PayError> {
    let client = reqwest::Client::new();

    let resp = client.get(MPP_SERVICES_URL).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(PayError::Http(format!(
            "MPP discovery returned {status}: {body}"
        )));
    }

    let body: MppServicesResponse = resp
        .json()
        .await
        .map_err(|e| PayError::Protocol(format!("failed to parse MPP discovery response: {e}")))?;

    Ok(body.services)
}

/// Search MPP services by keyword.
pub async fn search_mpp(query: &str) -> Result<Vec<MppService>, PayError> {
    let all = discover_mpp().await?;
    let q = query.to_lowercase();

    Ok(all
        .into_iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
                || s.categories.iter().any(|c| c.to_lowercase().contains(&q))
                || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Combined discovery (backwards-compat)
// ---------------------------------------------------------------------------

/// Fetch x402 services (alias for discover_x402).
pub async fn discover(
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<DiscoveredService>, PayError> {
    discover_x402(limit, offset).await
}

/// Search x402 services by keyword.
pub async fn search(query: &str) -> Result<Vec<DiscoveredService>, PayError> {
    let all = discover_x402(Some(100), None).await?;
    let q = query.to_lowercase();

    Ok(all
        .into_iter()
        .filter(|s| {
            let url_match = s.resource.to_lowercase().contains(&q);
            let accepts_desc = s
                .accepts
                .first()
                .and_then(|a| a.description.as_ref())
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false);
            let meta_desc = s
                .metadata
                .as_ref()
                .and_then(|m| m.description.as_ref())
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false);
            url_match || accepts_desc || meta_desc
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Format a token amount in smallest units to human-readable USD.
/// USDC has 6 decimals: "10000" → "$0.01".
pub fn format_usdc(amount_str: &str) -> String {
    let amount: u128 = amount_str.parse().unwrap_or(0);
    let whole = amount / 1_000_000;
    let frac = amount % 1_000_000;
    let frac_str = format!("{frac:06}");
    let trimmed = frac_str.trim_end_matches('0');
    let trimmed = if trimmed.is_empty() { "00" } else { trimmed };
    format!("${whole}.{trimmed}")
}

/// Format an MPP payment amount.
pub fn format_mpp_amount(amount_str: &str, decimals: u8) -> String {
    let amount: u128 = amount_str.parse().unwrap_or(0);
    let divisor = 10u128.pow(decimals as u32);
    let whole = amount / divisor;
    let frac = amount % divisor;
    let frac_str = format!("{frac:0>width$}", width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    let trimmed = if trimmed.is_empty() { "00" } else { trimmed };
    format!("${whole}.{trimmed}")
}
