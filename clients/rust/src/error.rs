#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("base URL is missing or invalid")]
    InvalidBase,
    #[error("bearer token is empty")]
    EmptyToken,
    #[error("HTTP {0}")]
    Http(u16),
    #[error("response exceeded the 64 KiB bound")]
    ResponseTooLarge,
    #[error("body was not valid JSON")]
    InvalidJson,
}

