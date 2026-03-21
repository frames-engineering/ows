use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayError {
    #[error("http: {0}")]
    Http(String),

    #[error("protocol: {0}")]
    Protocol(String),

    #[error("signing: {0}")]
    Signing(String),

    #[error("wallet: {0}")]
    Wallet(String),

    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl From<reqwest::Error> for PayError {
    fn from(e: reqwest::Error) -> Self {
        PayError::Http(e.to_string())
    }
}

impl From<ows_lib::OwsLibError> for PayError {
    fn from(e: ows_lib::OwsLibError) -> Self {
        PayError::Wallet(e.to_string())
    }
}

impl From<serde_json::Error> for PayError {
    fn from(e: serde_json::Error) -> Self {
        PayError::Protocol(format!("json: {e}"))
    }
}
