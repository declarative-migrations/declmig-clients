#![forbid(unsafe_code)]

use crate::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    pub base_url: String,
    pub bearer_token: Option<String>,
    pub max_response_bytes: usize,
}

/// A conservative HTTP(S) URL gate used before any client request.
pub fn is_valid_base_url(value: &str) -> bool {
    let rest = value.strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"));
    let Some(rest) = rest else { return false };
    let authority = rest.split('/').next().unwrap_or("");
    !authority.is_empty() && !authority.contains('@')
        && !rest.chars().any(|c| c.is_whitespace() || matches!(c, '?' | '#' | '\\'))
}

impl ClientConfig {
    pub fn from_env() -> Result<Self, ClientError> {
        let base = std::env::var("DECLMIG_API_BASE").map_err(|_| ClientError::InvalidBase)?;
        if !is_valid_base_url(&base) {
            return Err(ClientError::InvalidBase);
        }
        Ok(Self {
            base_url: base,
            bearer_token: std::env::var("DECLMIG_TOKEN").ok().filter(|v| !v.is_empty()),
            max_response_bytes: 64 * 1024,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::is_valid_base_url;

    #[test]
    fn rejects_credential_bearing_and_ambiguous_urls() {
        for value in ["", "ftp://example.test", "https://", "https://u:p@example.test",
                      "https://example.test?q=1", "https://example.test/#x",
                      "https://example.test/white space"] {
            assert!(!is_valid_base_url(value), "{value}");
        }
        assert!(is_valid_base_url("http://127.0.0.1:8080"));
        assert!(is_valid_base_url("https://api.example.test/v1"));
    }
}
