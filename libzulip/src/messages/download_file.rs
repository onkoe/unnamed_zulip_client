use tempfile::NamedTempFile;

use crate::{
    error::{FileError, ZulipError},
    Client,
};

impl Client {
    /// Downloads a file to a temporary path, then returns the path.
    #[tracing::instrument(skip(self))]
    pub async fn download_file<S>(&self, url: S) -> Result<NamedTempFile, ZulipError>
    where
        S: AsRef<str> + std::fmt::Debug + Send,
    {
        let url = self.api_url().join(url.as_ref())?;
        tracing::info!("downloading file... (url: {url}");

        let resp = self
            .auth(self.reqwest_client().get(url))
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("downloaded file successfully!");

        let temp_file = NamedTempFile::new()
            .map_err(|_| ZulipError::FileError(FileError::DownloadFailTempFile))?;
        let temp_file_path = temp_file.path();

        tracing::trace!("writing to disk at path `{}`...", temp_file_path.display());

        tokio::fs::write(temp_file_path, resp.bytes().await?)
            .await
            .map_err(|_| FileError::DownloadFailTempFile)?;

        tracing::trace!("file is now on disk!");
        Ok(temp_file)
    }
}
