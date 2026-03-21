use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// x402 protocol types
// ---------------------------------------------------------------------------

/// Payment option from an x402-enabled server's 402 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    pub scheme: String,
    /// CAIP-2 network identifier (e.g. "eip155:8453").
    pub network: String,
    /// Required amount in token's smallest unit (e.g. "10000" = $0.01 USDC).
    #[serde(alias = "maxAmountRequired")]
    pub amount: String,
    /// ERC-20 token contract address.
    pub asset: String,
    /// Recipient address.
    #[serde(alias = "payTo")]
    pub pay_to: String,
    #[serde(default = "default_timeout")]
    pub max_timeout_seconds: u64,
    /// Token metadata — must contain `name` and `version` for EIP-712 domain.
    #[serde(default)]
    pub extra: serde_json::Value,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
}

fn default_timeout() -> u64 {
    30
}

/// The 402 response body from an x402 server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402Response {
    #[serde(default)]
    pub x402_version: Option<u32>,
    pub accepts: Vec<PaymentRequirements>,
}

/// The signed payment payload sent back to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    pub x402_version: u32,
    pub scheme: String,
    pub network: String,
    pub payload: Eip3009Payload,
}

/// EIP-3009 `transferWithAuthorization` payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eip3009Payload {
    pub signature: String,
    pub authorization: Eip3009Authorization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip3009Authorization {
    pub from: String,
    pub to: String,
    pub value: String,
    pub valid_after: String,
    pub valid_before: String,
    pub nonce: String,
}

// ---------------------------------------------------------------------------
// x402 service discovery
// ---------------------------------------------------------------------------

/// A discovered x402 service from the Bazaar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredService {
    pub resource: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub x402_version: Option<u32>,
    #[serde(default)]
    pub accepts: Vec<PaymentRequirements>,
    #[serde(default)]
    pub metadata: Option<ServiceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub description: Option<String>,
    #[serde(default)]
    pub input: Option<serde_json::Value>,
    #[serde(default)]
    pub output: Option<serde_json::Value>,
}

/// Paginated response from the Bazaar discovery API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub items: Vec<DiscoveredService>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
    pub total: u64,
}

// ---------------------------------------------------------------------------
// MPP service discovery types
// ---------------------------------------------------------------------------

/// Response from `GET https://mpp.dev/api/services`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MppServicesResponse {
    pub version: u32,
    pub services: Vec<MppService>,
}

/// A service from the MPP directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MppService {
    pub id: String,
    pub name: String,
    #[serde(alias = "url")]
    pub service_url: String,
    pub description: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub endpoints: Vec<MppEndpoint>,
}

/// A single endpoint within an MPP service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MppEndpoint {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub payment: Option<MppPayment>,
}

/// Payment info for an MPP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MppPayment {
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub decimals: Option<u8>,
    #[serde(default)]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// MoonPay types
// ---------------------------------------------------------------------------

/// Request body for MoonPay `deposit_create` tool.
#[derive(Debug, Clone, Serialize)]
pub struct MoonPayDepositRequest {
    pub name: String,
    pub wallet: String,
    pub chain: String,
    pub token: String,
}

/// Response from MoonPay `deposit_create` tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayDepositResponse {
    pub id: String,
    pub destination_wallet: String,
    pub destination_chain: String,
    pub customer_token: String,
    pub deposit_url: String,
    pub wallets: Vec<DepositWallet>,
    pub instructions: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositWallet {
    pub address: String,
    pub chain: String,
    pub qr_code: String,
}

/// Request body for MoonPay `token_balance_list` tool.
#[derive(Debug, Clone, Serialize)]
pub struct MoonPayBalanceRequest {
    pub wallet: String,
    pub chain: String,
}

/// Response from MoonPay `token_balance_list` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct MoonPayBalanceResponse {
    pub items: Vec<TokenBalance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBalance {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub chain: String,
    pub decimals: u32,
    pub balance: BalanceInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceInfo {
    pub amount: f64,
    pub value: f64,
    pub price: f64,
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of a successful `ows pay` operation.
#[derive(Debug, Clone)]
pub struct PayResult {
    /// HTTP status of the paid response.
    pub status: u16,
    /// Response body.
    pub body: String,
    /// How much was paid (human-readable, e.g. "$0.01").
    pub amount_display: String,
    /// Network the payment was made on.
    pub network: String,
}

/// Result of `ows fund`.
#[derive(Debug, Clone)]
pub struct FundResult {
    /// Deposit ID for tracking.
    pub deposit_id: String,
    /// MoonPay deposit URL (shareable, opens in browser).
    pub deposit_url: String,
    /// Multi-chain deposit addresses.
    pub wallets: Vec<(String, String)>,
    /// Human-readable instructions.
    pub instructions: String,
}
