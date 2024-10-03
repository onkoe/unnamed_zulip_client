use std::collections::HashMap;

use crate::{
    error::{MessageError, ResponseError, ZulipError},
    Client,
};

use super::emoji_reaction::ReactionType;

impl Client {
    /// Given a message ID, return the message object.
    ///
    /// Additionally, a `raw_content` field is included. This field is useful
    /// for clients that primarily work with HTML-rendered messages, but may
    /// need to occasionally fetch the message's raw Markdown (e.g. for view
    /// source or prefilling a message edit textarea).
    ///
    /// Note: you probably want `apply_markdown` to be `false`, as this decides
    /// if the returned message will be in rendered (HTML) form or if it'll
    /// keep the user's original `markdown` (`false`).
    ///
    /// TODO: fix when not broken: https://github.com/zulip/zulip/issues/31832
    pub async fn fetch_single_message(
        &self,
        msg_id: u64,
        apply_markdown: bool,
    ) -> Result<SingleMessageResponse, ZulipError> {
        let url = self.api_url().join(format!("messages/{msg_id}").as_str())?;

        let mut parameters = HashMap::new();
        parameters.insert("apply_markdown", apply_markdown.to_string());

        let resp = self
            .auth(self.reqwest_client().get(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?
            .json::<SingleMessageResponse>()
            .await?;

        if let Some(error) = resp.error {
            error.warn_ignored();
            return Err(MessageError::SingleMessageFetchFailed {
                msg_id,
                error: error.to_string(),
            }
            .into());
        }

        Ok(resp)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SingleMessageResponse {
    /// A potential error code.
    #[serde(flatten)]
    pub error: Option<ResponseError>,
    /// An object containing details of the message.
    pub message: Message,
}

/// A representation of a message. Contains most important details.
#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct Message {
    /// The URL of the message sender's avatar.
    pub avatar_url: Option<String>,
    /// A Zulip "client" string, describing what Zulip client sent the message.
    pub client: String,
    /// The content/body of the message.
    pub content: String,
    /// The HTTP content_type for the message content. This will be `text/html`
    /// or `text/x-markdown`, depending on whether `apply_markdown` was set.
    pub content_type: String,
    // `display_recipient: DisplayRecipient`, // TODO: not sure what the Users variant should contain...
    /// An array of changes made to the message.
    pub edit_history: Option<Vec<MessageEdit>>,
    /// The unique message ID. Messages should always be displayed sorted by ID.
    pub id: u64,
    /// Whether the message is a `/me` status message
    pub is_me_message: bool,
    ///The UNIX timestamp for when the message was last edited, in UTC seconds.
    ///
    /// Not present if the message has never been edited.
    pub last_edit_timestamp: Option<u64>,
    /// Data on any reactions to the message.
    pub reactions: Option<Vec<Emoji>>,
    /// A unique ID for the set of users receiving the message (either a
    /// channel or group of users). Useful primarily for hashing.
    pub recipient_id: u64,
    /// The Zulip API email address of the message's sender.
    pub sender_email: String,
    /// The full name of the message's sender.
    pub sender_full_name: String,
    /// The user ID of the message's sender.
    pub sender_id: u64,
    /// A string identifier for the realm the sender is in. Unique only within
    /// the context of a given Zulip server.
    ///
    /// E.g. on `example.zulip.com`, this will be `example`.
    pub sender_realm_str: String,
    /// Only present for channel messages; the ID of the channel.
    pub stream_id: Option<u64>,
    /// warning! this will change its name eventually as per the docs.
    pub subject: String,
    pub timestamp: u64,
    pub topic_links: Vec<Link>,
    #[serde(rename = "type")]
    pub typ: MessageType,
    pub flags: Vec<String>, // FIXME: this should use a `MessageFlags` type later on
}

#[derive(Debug, serde::Deserialize)]
pub enum DisplayRecipient {
    ChannelName(String),
    Users {
        // TODO
    },
}

/// Documents the changes in a previous edit made to the message.
#[derive(Debug, serde::Deserialize)]
pub struct MessageEdit {
    pub prev_content: Option<String>,
    pub prev_rendered_content: Option<String>,
    pub prev_stream: Option<u64>,
    pub prev_topic: Option<u64>,
    pub stream: Option<u64>,
    pub timestamp: u64,
    pub topic: Option<String>,
    pub user_id: Option<u64>,
}

/// Use this to select which emoji to add.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, serde::Deserialize)]
pub struct Emoji {
    /// The target emoji's human-readable name.
    pub emoji_name: String,
    /// A unique identifier, defining the specific emoji codepoint requested,
    /// within the namespace of the reaction_type.
    ///
    /// For most API clients, you won't need this, but it's important for Zulip
    /// apps to handle rare corner cases when adding/removing votes on an emoji
    /// reaction added previously by another user.
    pub emoji_code: Option<String>,
    /// Indicates the type of emoji. Each emoji reaction_type has an
    /// independent namespace for values of emoji_code.
    ///
    /// If an API client is adding/removing a vote on an existing reaction, it
    /// should pass this parameter using the value the server provided for the
    /// existing reaction for specificity.
    pub reaction_type: Option<ReactionType>,
    /// The ID of the user who added the reaction.
    pub user_id: u64,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Stream,
    Private,
}
