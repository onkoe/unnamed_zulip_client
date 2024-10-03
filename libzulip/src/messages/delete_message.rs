use crate::{
    error::{MessageError, ResponseError, ZulipError},
    Client,
};

impl Client {
    /// Permanently delete a message.
    ///
    /// This endpoint is only available to organization administrators.
    ///
    /// For more, see: https://zulip.com/help/delete-a-message#delete-a-message-completely
    pub async fn delete_message(&self, msg_id: u64) -> Result<(), ZulipError> {
        let url = self.api_url().join(&format!("messages/{msg_id}"))?;

        let resp = self
            .auth(self.reqwest_client().delete(url))
            .send()
            .await?
            .error_for_status()?
            .json::<DeletedMessageResponse>()
            .await?;

        if let Some(error) = resp.error {
            error.warn_ignored();
            return Err(MessageError::DeletionFailed {
                id: msg_id,
                error: error.to_string(),
            }
            .into());
        }

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DeletedMessageResponse {
    #[serde(flatten)]
    pub error: Option<ResponseError>,
}
