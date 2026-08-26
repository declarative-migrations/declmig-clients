#![forbid(unsafe_code)]

use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::types::Health;

/// Transport-agnostic client. Callers inject HTTP via `get_json`.
pub struct Client {
    config: ClientConfig,
}

impl Client {
    pub fn new(config: ClientConfig) -> Result<Self, ClientError> {
        if config.base_url.trim().is_empty() {
            return Err(ClientError::InvalidBase);
        }
        Ok(Self { config })
    }

    pub fn health_path(&self) -> String {
        format!("{}/v1/health", self.config.base_url.trim_end_matches('/'))
    }

    pub fn authorize<'a>(&'a self, token_override: Option<&'a str>) -> Result<Option<&'a str>, ClientError> {
        let token = token_override.or(self.config.bearer_token.as_deref());
        if let Some(value) = token {
            if value.trim().is_empty() {
                return Err(ClientError::EmptyToken);
            }
        }
        Ok(token)
    }

    pub fn decode_health(&self, body: &[u8]) -> Result<Health, ClientError> {
        if body.len() > self.config.max_response_bytes {
            return Err(ClientError::ResponseTooLarge);
        }
        serde_json::from_slice(body).map_err(|_| ClientError::InvalidJson)
    }
}

