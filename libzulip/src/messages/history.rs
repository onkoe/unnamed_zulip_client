use std::{collections::HashMap, path::Path};

use tempfile::NamedTempFile;

use crate::{
    error::{FileError, MessageError, ZulipError},
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

    /// Downloads a file to a temporary path, then returns the path.
    #[tracing::instrument(skip(self))]
    pub async fn download_file<S: AsRef<str> + std::fmt::Debug>(
        &self,
        url: S,
    ) -> Result<NamedTempFile, ZulipError> {
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

    #[tracing::instrument(skip(self))]
    pub async fn edit_message(
        &self,
        edited_message: EditedMessage,
    ) -> Result<EditedMessageResponse, ZulipError> {
        let url = self
            .api_url()
            .join(&format!("messages/{}", edited_message.message_id))?;

        let mut parameters = HashMap::new();

        if let Some(topic) = edited_message.topic {
            parameters.insert("topic", topic);
        }

        // FIXME: propogate_mode should be given with editedmessage as an enum
        parameters.insert("propagate_mode", "change_one".into());

        if let Some(noti_old) = edited_message.send_notification_to_old_thread {
            parameters.insert("send_notification_to_old_thread", noti_old.to_string());
        }
        if let Some(noti_new) = edited_message.send_notification_to_new_thread {
            parameters.insert("send_notification_to_new_thread", noti_new.to_string());
        }
        if let Some(content) = edited_message.content {
            parameters.insert("content", content);
        }
        if let Some(stream_id) = edited_message.stream_id {
            parameters.insert("stream_id", stream_id.to_string());
        }

        let resp = self
            .auth(self.reqwest_client().patch(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("message edited successfully!");

        Ok(serde_json::from_str::<EditedMessageResponse>(
            &resp.text().await?,
        )?)
    }

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

/// A representation of an uploaded file.
#[derive(Debug, serde::Deserialize)]
pub struct UploadFileResponse {
    pub url: String,
    pub filename: String,
}

// TODO: refactor this as an enum to hold `propogate_mode`'s invariants
#[derive(Debug, serde::Deserialize)]
pub struct EditedMessage {
    /// The ID of the message you wish to update.
    pub message_id: u64,
    /// If you wish to request changing the topic, set this to the new
    /// topic name.
    pub topic: Option<String>,
    /// Whether to send an automated message to the old topic to notify users
    /// where the messages were moved to.
    pub send_notification_to_old_thread: Option<bool>,
    /// Whether to send an automated message to the new topic to notify users
    /// where the messages came from.
    pub send_notification_to_new_thread: Option<bool>,
    /// The updated content of the target message.
    pub content: Option<String>,
    /// The channel ID to move the message(s) to, to request moving messages to
    /// another channel.
    pub stream_id: Option<u64>,
}

/// The edit mode for a channel, topic, or message: Which message(s) should be
/// edited.
///
/// This is always `message` (`Message`) when editing those.
pub enum PropagateMode {
    /// The target message and all following messages.
    ChangeLater,
    /// Only the target message.
    ChangeOne,
    /// All messages in this topic.
    ChangeAll,
}

#[derive(Debug, serde::Deserialize)]
pub struct EditedMessageResponse {
    /// Details on all files uploaded by the acting user whose only references
    /// were removed when editing this message
    pub detached_uploads: Vec<DetachedUpload>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DetachedUpload {
    /// The unique ID for the attachment.
    pub id: u64,
    /// Name of the uploaded file.
    pub name: String,
    /// A representation of the path of the file within the repository of
    /// user-uploaded files. If the path_id of a file is `{realm_id}/ab/cdef/temp_file.py`,
    /// its URL will be: `{server_url}/user_uploads/{realm_id}/ab/cdef/temp_file.py`.
    pub path_id: String,
    /// Size of the file in bytes.
    pub size: u64,
    /// Time when the attachment was uploaded as a UNIX timestamp multiplied by
    /// 1000 (matching the format of getTime() in JavaScript).
    pub create_time: u64,
    /// Contains basic details on any Zulip messages that have been sent
    /// referencing this uploaded file. This includes messages sent by any user
    /// in the Zulip organization who sent a message containing a link to the
    /// uploaded file.
    pub messages: Vec<BasicMessageRepresentation>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BasicMessageRepresentation {
    /// Time when the message was sent as a UNIX timestamp multiplied by 1000
    /// (matching the format of getTime() in JavaScript).
    pub date_sent: u64,
    /// The unique message ID.
    pub id: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeletedMessageResponse {
    pub code: Option<String>,
}
