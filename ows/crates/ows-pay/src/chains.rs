/// Chain mapping between OWS, MoonPay, and x402 identifiers.
///
/// This is the single source of truth for chain compatibility across the three systems.

/// A supported chain with its identifiers across all systems.
#[derive(Debug, Clone)]
pub struct ChainMapping {
    /// Human-readable name.
    pub name: &'static str,
    /// OWS chain string (e.g. "ethereum", "base").
    pub ows_chain: &'static str,
    /// CAIP-2 identifier used by x402 (e.g. "eip155:8453").
    pub caip2: &'static str,
    /// MoonPay currency code for USDC buy (e.g. "usdc_base").
    pub moonpay_token: &'static str,
    /// MoonPay chain name for balance queries (e.g. "base").
    pub moonpay_chain: &'static str,
    /// USDC contract address on this chain (empty if unknown).
    pub usdc_address: &'static str,
    /// USDC decimals (6 for all current USDC deployments).
    pub usdc_decimals: u8,
}

/// All chains where OWS + MoonPay + x402 overlap (the "golden path" chains).
pub const SUPPORTED_CHAINS: &[ChainMapping] = &[
    ChainMapping {
        name: "Base",
        ows_chain: "base",
        caip2: "eip155:8453",
        moonpay_token: "usdc_base",
        moonpay_chain: "base",
        usdc_address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        usdc_decimals: 6,
    },
    ChainMapping {
        name: "Ethereum",
        ows_chain: "ethereum",
        caip2: "eip155:1",
        moonpay_token: "usdc",
        moonpay_chain: "ethereum",
        usdc_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        usdc_decimals: 6,
    },
    ChainMapping {
        name: "Polygon",
        ows_chain: "polygon",
        caip2: "eip155:137",
        moonpay_token: "usdc_polygon",
        moonpay_chain: "polygon",
        usdc_address: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
        usdc_decimals: 6,
    },
    ChainMapping {
        name: "Arbitrum",
        ows_chain: "arbitrum",
        caip2: "eip155:42161",
        moonpay_token: "usdc_arbitrum",
        moonpay_chain: "arbitrum",
        usdc_address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
        usdc_decimals: 6,
    },
    ChainMapping {
        name: "Optimism",
        ows_chain: "optimism",
        caip2: "eip155:10",
        moonpay_token: "usdc_optimism",
        moonpay_chain: "optimism",
        usdc_address: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85",
        usdc_decimals: 6,
    },
];

/// Testnet chains (for development / demos).
pub const TESTNET_CHAINS: &[ChainMapping] = &[ChainMapping {
    name: "Base Sepolia",
    ows_chain: "base-sepolia",
    caip2: "eip155:84532",
    moonpay_token: "",
    moonpay_chain: "base-sepolia",
    usdc_address: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
    usdc_decimals: 6,
}];

/// Default chain for payments (Base — lowest gas, broadest x402 support).
pub const DEFAULT_CHAIN: &ChainMapping = &SUPPORTED_CHAINS[0]; // Base

/// Look up a chain by its CAIP-2 network identifier (from an x402 402 response).
pub fn chain_by_caip2(caip2: &str) -> Option<&'static ChainMapping> {
    SUPPORTED_CHAINS
        .iter()
        .chain(TESTNET_CHAINS.iter())
        .find(|c| c.caip2 == caip2)
}

/// Look up a chain by its MoonPay chain name.
pub fn chain_by_moonpay(moonpay_chain: &str) -> Option<&'static ChainMapping> {
    SUPPORTED_CHAINS
        .iter()
        .chain(TESTNET_CHAINS.iter())
        .find(|c| c.moonpay_chain == moonpay_chain)
}

/// Look up a chain by human-readable name (case-insensitive).
pub fn chain_by_name(name: &str) -> Option<&'static ChainMapping> {
    let lower = name.to_lowercase();
    SUPPORTED_CHAINS
        .iter()
        .chain(TESTNET_CHAINS.iter())
        .find(|c| {
            c.name.to_lowercase() == lower || c.ows_chain == lower || c.moonpay_chain == lower
        })
}
