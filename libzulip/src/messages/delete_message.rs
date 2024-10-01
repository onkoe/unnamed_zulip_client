use crate::{
    error::{MessageError, ZulipError},
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
            .error_for_status()?;

        let parsed_resp = serde_json::from_str::<DeletedMessageResponse>(&resp.text().await?)?;

        if let Some(error_code) = parsed_resp.code {
            return Err(MessageError::DeletionFailed {
                id: msg_id,
                error_code,
            }
            .into());
        }

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DeletedMessageResponse {
    pub code: Option<String>,
}
