//! `ows-pay` — self-contained x402 payment client for the Open Wallet Standard.
//!
//! This crate is independent from OWS core logic. It only uses `ows-lib` for
//! wallet access and EIP-712 signing. Everything else — HTTP, discovery,
//! protocol parsing, credential construction — is self-contained here.
//!
//! # Quick start
//!
//! ```ignore
//! // Fund a wallet via MoonPay
//! let result = ows_pay::fund::fund("0xABC...", 5.0, None, None).await?;
//! open_browser(&result.checkout_url);
//!
//! // Discover x402 services
//! let services = ows_pay::discovery::search("web scraping").await?;
//!
//! // Pay for an API call
//! let result = ows_pay::x402::pay("my-wallet", "", "https://api.example.com/search", "GET", None).await?;
//! println!("{}", result.body);
//! ```

pub mod chains;
pub mod discovery;
pub mod error;
pub mod fund;
pub mod types;
pub mod x402;

pub use error::PayError;
