use std::collections::HashMap;

use crate::{
    error::{MessageError, ResponseError, ZulipError},
    Client,
};

impl Client {
    pub async fn add_emoji_reaction(
        &self,
        msg_id: u64,
        selector: EmojiSelector,
    ) -> Result<(), ZulipError> {
        let url = self
            .api_url()
            .join(format!("messages/{msg_id}/reactions").as_str())?;

        // create parameters
        let parameters = selector.make_parameters();

        // send the request
        let resp = self
            .auth(self.reqwest_client().post(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?
            .json::<EmojiReactionResponse>()
            .await?;

        tracing::trace!("added emoji reaction successfully!");

        if let Some(error) = resp.error {
            return Err(MessageError::AddEmojiFailed {
                msg_id,
                emoji_name: selector.emoji_name,
                error: error.to_string(),
            }
            .into());
        }

        Ok(())
    }

    pub async fn remove_emoji_reaction(
        &self,
        msg_id: u64,
        selector: EmojiSelector,
    ) -> Result<(), ZulipError> {
        let url = self
            .api_url()
            .join(format!("messages/{msg_id}/reactions").as_str())?;

        // create parameters
        let parameters = selector.make_parameters();

        // send the request
        let resp = self
            .auth(self.reqwest_client().delete(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?
            .json::<EmojiReactionResponse>()
            .await?;

        tracing::trace!("removed emoji reaction successfully!");

        if let Some(error) = resp.error {
            error.warn_ignored();
            return Err(MessageError::RemoveEmojiFailed {
                msg_id,
                emoji_name: selector.emoji_name,
                error: error.to_string(),
            }
            .into());
        }

        Ok(())
    }
}

/// Use this to select which emoji to add.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct EmojiSelector {
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
}

impl EmojiSelector {
    /// Creates a new `EmojiSelector` from just an emoji name. Often, clients
    /// use exclusively the name for these, so this is good shorthand.
    ///
    /// See the type docs for more info.
    pub fn new_from_name<S>(name: S) -> Self
    where
        S: AsRef<str> + std::fmt::Debug,
    {
        Self {
            emoji_name: name.as_ref().into(),
            emoji_code: None,
            reaction_type: None,
        }
    }

    fn make_parameters(&self) -> HashMap<&str, String> {
        // urlencode the emoji name
        let emoji_name = urlencoding::encode(&self.emoji_name).to_string();

        let mut parameters = HashMap::new();
        parameters.insert("emoji_name", emoji_name);

        if let Some(emoji_code) = self.emoji_code.clone() {
            parameters.insert("emoji_code", emoji_code);
        }

        if let Some(reaction_type) = self.reaction_type.clone() {
            parameters.insert("reaction_type", reaction_type.to_string());
        }

        parameters
    }
}

/// Indicates the type of emoji. Each emoji `reaction_type` has an independent
/// namespace for values of `emoji_code`.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionType {
    /// In this namespace, `emoji_code` will be a dash-separated hex encoding
    /// of the sequence of Unicode codepoints that define this emoji in the
    /// Unicode specification.
    UnicodeEmoji,
    /// In this namespace, `emoji_code` will be the ID of the uploaded custom
    /// emoji.
    RealmEmoji,
    /// These are special emoji included with Zulip. In this namespace,
    /// `emoji_code` will be the name of the emoji (e.g. "zulip").
    ZulipExtraEmoji,
}

impl std::fmt::Display for ReactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ReactionType::UnicodeEmoji => f.write_str("unicode_emoji"),
            ReactionType::RealmEmoji => f.write_str("realm_emoji"),
            ReactionType::ZulipExtraEmoji => f.write_str("zulip_extra_emoji"),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EmojiReactionResponse {
    #[serde(flatten)]
    pub error: Option<ResponseError>,
}
