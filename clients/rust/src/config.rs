#![forbid(unsafe_code)]

use crate::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    pub base_url: String,
    pub bearer_token: Option<String>,
    pub max_response_bytes: usize,
}

impl ClientConfig {
    pub fn from_env() -> Result<Self, ClientError> {
        let base = std::env::var("DECLMIG_API_BASE").map_err(|_| ClientError::InvalidBase)?;
        if base.trim().is_empty() {
            return Err(ClientError::InvalidBase);
        }
        Ok(Self {
            base_url: base,
            bearer_token: std::env::var("DECLMIG_TOKEN").ok().filter(|v| !v.is_empty()),
            max_response_bytes: 64 * 1024,
        })
    }
}

