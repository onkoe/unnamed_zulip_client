use std::collections::HashMap;

use crate::{
    error::{MessageError, ResponseError, ZulipError},
    Client,
};

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn send_message(&self, msg: &Message) -> Result<MessageResponse, ZulipError> {
        let url = self.api_url().join("messages").unwrap();

        // make the parameters
        let parameters = msg.make_parameters();

        // post the request and grab its response
        let resp = self
            .auth(self.reqwest_client().post(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?
            .json::<MessageResponse>()
            .await?;

        if let Some(error) = resp.error {
            return Err(MessageError::SendFailed {
                content: msg.content(),
                error: error.to_string(),
            }
            .into());
        }

        tracing::trace!("sent msg successfully!");

        // try to parse the reply out
        Ok(resp)
    }
}

/// The message being sent.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Direct {
        to: DirectMessageTarget,
        content: String,
        queue_id: String, // TODO
        local_id: String, // TODO
    },
    Stream {
        content: String,
        topic: String,
        queue_id: String, // TODO
        local_id: String, // TODO
    },
    Channel {
        to: ChannelMessageTarget,
        content: String,
        topic: String,
        queue_id: String, // TODO
        local_id: String, // TODO
    },
}

impl Message {
    /// Creates the parameters for this function for use
    #[tracing::instrument]
    fn make_parameters(&self) -> HashMap<&str, String> {
        // gather message info (these are all required)
        let mut parameters = HashMap::from([
            ("local_id", self.local_id()),
            ("queue_id", self.queue_id()),
            ("content", self.content()),
            ("type", self.typ().into()),
        ]);

        // grab the optionals and add them if we got em
        if let Some(to) = self.to() {
            parameters.insert("to", to);
        }
        if let Some(topic) = self.topic() {
            parameters.insert("topic", topic);
        }

        // return our new list of parameters
        parameters
    }

    fn typ(&self) -> &'static str {
        match *self {
            Message::Direct { .. } => "direct",
            Message::Stream { .. } => "stream",
            Message::Channel { .. } => "channel",
        }
    }

    fn to(&self) -> Option<String> {
        match *self {
            Message::Channel { ref to, .. } => match to {
                ChannelMessageTarget::Name(s) => Some(s.clone()),
                ChannelMessageTarget::Id(number) => Some(number.to_string()),
            },
            Message::Direct { ref to, .. } => match to {
                DirectMessageTarget::Ids(vec) => serde_json::to_string(vec).ok(),
                DirectMessageTarget::Emails(vec) => serde_json::to_string(vec).ok(),
            },
            Message::Stream { .. } => None,
        }
    }

    fn content(&self) -> String {
        match *self {
            Self::Direct { ref content, .. }
            | Self::Stream { ref content, .. }
            | Self::Channel { ref content, .. } => content.clone(),
        }
    }

    fn topic(&self) -> Option<String> {
        match *self {
            Message::Direct { .. } => None,
            Message::Stream { ref topic, .. } | Message::Channel { ref topic, .. } => {
                Some(topic.clone())
            }
        }
    }

    fn queue_id(&self) -> String {
        match *self {
            Self::Direct { ref queue_id, .. }
            | Self::Stream { ref queue_id, .. }
            | Self::Channel { ref queue_id, .. } => queue_id.clone(),
        }
    }

    fn local_id(&self) -> String {
        match *self {
            Self::Direct { ref local_id, .. }
            | Self::Stream { ref local_id, .. }
            | Self::Channel { ref local_id, .. } => local_id.clone(),
        }
    }
}

/// The channel(s) a message will be sent to.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, serde::Deserialize)]
pub enum ChannelMessageTarget {
    Name(String),
    Id(u64),
}

/// The person(s) a message will be sent to.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, serde::Deserialize)]
pub enum DirectMessageTarget {
    Ids(Vec<u64>),
    Emails(Vec<String>),
}

#[derive(Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct MessageResponse {
    pub id: u64,
    pub automatic_new_visibility_policy: Option<u8>,

    #[serde(flatten)]
    pub error: Option<ResponseError>,
    pub stream: Option<String>,
}
