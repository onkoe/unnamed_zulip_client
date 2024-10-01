use std::path::Path;

use crate::{
    error::{FileError, ZulipError},
    Client,
};

impl Client {
    /// Attempts to upload a file to Zulip.
    #[tracing::instrument(skip(self))]
    pub async fn upload_file<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        path: P,
    ) -> Result<UploadFileResponse, ZulipError> {
        let path = path.as_ref().to_path_buf();
        let path_str = path.display().to_string();

        let file_name = {
            let p = path.to_path_buf();

            p.file_name()
                .ok_or(ZulipError::FileError(FileError::FileNameNotFound(
                    path_str.clone(),
                )))?
                .to_string_lossy()
                .to_string()
                .clone()
        };

        tracing::trace!("checking if file exists...");
        // make sure we have a file at all
        if tokio::fs::try_exists(&path).await.is_err() {
            return Err(ZulipError::FileError(FileError::FileNotFound(
                path_str.clone(),
            )));
        }
        tracing::trace!("file exists. making url...");

        // make the url
        tracing::info!("creating url...");
        let url = self.api_url().join("user_uploads").unwrap(); // FIXME(bray/perf): api/v1/tus instead?
        tracing::trace!("url created! uploading...");

        // upload that mf
        let resp = self
            .auth(self.reqwest_client().post(url))
            .multipart(
                reqwest::multipart::Form::new()
                    .file(file_name, path.clone())
                    .await
                    .map_err(move |_| FileError::AttachSerializeFailed(path_str.clone()))?,
            )
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("uploaded file successfully!");

        tracing::trace!("parsing reply...");
        // try to parse the reply out
        Ok(serde_json::from_str::<UploadFileResponse>(
            &resp.text().await?,
        )?)
    }
}

/// A representation of an uploaded file.
#[derive(Debug, serde::Deserialize)]
pub struct UploadFileResponse {
    pub url: String,
    pub filename: String,
}
