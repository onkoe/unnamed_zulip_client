use std::collections::HashMap;

use crate::{error::ZulipError, Client};

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn send_message(&self, msg: &Message) -> Result<MessageResponse, ZulipError> {
        let url = self.api_url().join("messages").unwrap();

        // these constitute the msg we'll send
        let mut parameters = HashMap::new();
        parameters.insert("type", msg.typ());

        // gather message info
        let to = msg.to();
        let content = msg.content();
        let topic = msg.topic();
        let queue_id = msg.queue_id();
        let local_id = msg.local_id();

        if let Some(place) = &to {
            parameters.insert("to", place);
        }

        parameters.insert("content", &content);

        if let Some(top) = &topic {
            parameters.insert("topic", top);
        }

        parameters.insert("queue_id", &queue_id);
        parameters.insert("local_id", &local_id);

        // post the request and grab its response
        let resp = self
            .auth(self.reqwest_client().post(url))
            .form(&parameters)
            .send()
            .await?
            .error_for_status()?;

        tracing::trace!("sent msg successfully!");

        // try to parse the reply out
        Ok(serde_json::from_str::<MessageResponse>(
            &resp.text().await?,
        )?)
    }
}

/// The message being sent.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ChannelMessageTarget {
    Name(String),
    Id(u64),
}

/// The person(s) a message will be sent to.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum DirectMessageTarget {
    Ids(Vec<u64>),
    Emails(Vec<String>),
}

#[derive(Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct MessageResponse {
    pub id: u64,
    pub automatic_new_visibility_policy: Option<u8>,

    // undocumented optionals :p
    pub result: Option<String>,
    pub msg: Option<String>,
    pub code: Option<String>,
    pub stream: Option<String>,
}
