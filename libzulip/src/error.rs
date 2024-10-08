use pisserror::Error;
use std::error::Error;

/// An error that potentially appears in a response.
///
/// ... ok, you! maintainer! this is an error that we check for in any response.
/// it should ALWAYS be `serde::flatten`, since responses don't contain an error
/// cool as that would be.
///
/// you should also make it an `Option<ResponseError>` in the response type.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, serde::Deserialize)]
pub struct ResponseError {
    code: String,
    msg: String,
    ignored_parameters_unsupported: Option<Vec<String>>,
}

impl ResponseError {
    /// Creates a `tracing::warn!` if any of the given parameters were ignored.
    ///
    /// Please run this function if you get this type, as it shows the user
    /// more about what went wrong internally.
    #[tracing::instrument]
    pub(crate) fn warn_ignored(&self) {
        if let Some(ignored) = &self.ignored_parameters_unsupported {
            tracing::warn!("some given parameters were ignored! these are: {ignored:#?}");
        }
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "err({}): {}", self.code, self.msg)
    }
}

/// The main error type for this crate.
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
    #[error("{_0}")]
    MessageError(#[from] MessageError),
}

/// Errors from file upload/download.
#[derive(Clone, Debug, Error)]
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

/// Errors when performing messaging tasks.
#[derive(Clone, Debug, Error)]
pub enum MessageError {
    #[error("Failed to send the given message. content: `{content}`. {error}")]
    SendFailed { content: String, error: String },

    #[error("Failed to delete the message with ID `{id}`. {error}")]
    DeletionFailed { id: u64, error: String },

    #[error(
        "Couldn't add an emoji reaction to message `{msg_id}` with emoji name `{emoji_name}`. {error}"
    )]
    AddEmojiFailed {
        msg_id: u64,
        emoji_name: String,
        error: String,
    },

    #[error(
        "Couldn't remove an emoji reaction to message `{msg_id}` with emoji name `{emoji_name}`. {error}"
    )]
    RemoveEmojiFailed {
        msg_id: u64,
        emoji_name: String,
        error: String,
    },

    #[error("Failed to upload the given file. (path: {path}, {error})")]
    FileUploadFailed { path: String, error: String },

    #[error("Failed to fetch the message with ID `{msg_id}`. {error}")]
    SingleMessageFetchFailed { msg_id: u64, error: String },

    #[error("The server failed to render the following message: `{content}`. {error}")]
    RenderMessageFailed { content: String, error: String },
}
