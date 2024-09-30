use pisserror::Error;
use std::error::Error;

#[derive(Debug, Error)]
pub enum ZulipError {
    #[error("{_0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serialization of an object failed. Additional information is unavailable with this error, so please check your logs.")]
    SerdeJsonError(#[from] serde_json::Error),
}
