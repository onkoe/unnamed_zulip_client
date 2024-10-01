use pisserror::Error;
use std::error::Error;

#[derive(Debug, Error)]
pub enum ZulipError {
    #[error("Error with API request. err: {_0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serialization of an object failed. err: {_0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("An error occured involving file transfer. err: {_0}")]
    FileError(#[from] FileError),
    #[error("The given URL didn't parse correctly. err: {_0}")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("The given file was not found on disk. (path: `{_0}`)")]
    FileNotFound(String),
    #[error(
        "The file you attempted to upload was too large. (max: {max} bytes, given: {given} bytes.)"
    )]
    FileTooLarge { max: u64, given: u64 },
    #[error("Failed to create temporary file for download! Permissions might be messed up...")]
    DownloadFailTempFile,
    #[error("Unable to determine file name for the given path. (path: `{_0}`)")]
    FileNameNotFound(String),
    #[error("Failed to attach file to request. (path: `{_0}`)")]
    AttachSerializeFailed(String),
}