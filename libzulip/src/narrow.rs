//! Contains an implementation of Zulip's `Narrow` type, useful for creating a
//! set of filters on various Zulip constructs.

/// A list of [`Narrow`]s.
///
/// Or, in slightly cooler words, a query that hasn't been run yet.
pub type NarrowList = Vec<Narrow>;

/// A Narrow is a set of filters for Zulip messages that can be based on many
/// different factors, such as the sender, channel, topic, search keywords, etc...
///
/// Narrows are used in various places in the Zulip API - most importantly, in
/// the API for fetching messages.
pub struct Narrow {
    kind: NarrowKind,
    negation: NarrowNegation,
}

impl Narrow {
    /// Constructs a new `Narrow` given a kind and negation.
    ///
    /// - `kind`: A [`NarrowKind`]. This construct indicates the "reason"
    ///   behind the search. For example, it's intuitive that someone searching
    ///   in direct messages would use a `NarrowKind::DirectMessage`.
    /// - `negation`: A [`NarrowNegation`]. These indicate whether or not the
    ///   query should use the opposite of the given `kind`. So, it someone
    ///   uses the `DirectMessage` operator, but sets `negation` to `Negated`,
    ///   the search will include everything *except* direct messages.
    pub fn new(kind: NarrowKind, negation: NarrowNegation) -> Self {
        Narrow { kind, negation }
    }

    /// Grabs this `Narrow`'s [`NarrowKind`].
    pub fn kind(&self) -> NarrowKind {
        self.kind.clone()
    }

    /// Grabs this `Narrow`'s [`NarrowNegation`].
    pub fn negation(&self) -> NarrowNegation {
        self.negation.clone()
    }
}

/// Whether or not a `Narrow`'s kind will be negated in the query.
///
/// In other words, if this holds the `Negated` variant, then, the opposite of
/// the given condition will be used.
///
/// Ex: `{NarrowKind::Keyword("hi"), Negated}` will find messages that do NOT
/// contain "hi".
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum NarrowNegation {
    /// The narrow is left as-is. It is unchanged. This is the default value.
    Normal,
    /// Used to negate (perform the opposite of) the paired `NarrowKind`.
    Negated,
}

/// A certain "kind" of [`Narrow`] used within the query.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum NarrowKind {
    /// Zulip lets you search messages and topics by keyword.
    ///
    /// Ex: `new logo` looks for messages with both "new" and "logo" in the
    /// message or its topic.
    ///
    /// You may use double quotes to specify that you wish to see an exact
    /// phrase. For instance, `"new logo"` looks for messages that contain the
    /// entire phrase.
    ///
    /// ## Details
    ///
    /// - Keywords are case-**in**sensitive. Ex: `wave` matches `Wave` and `WAVE`.
    /// - Zulip looks for keywords with the same word stem. Ex: `wave` looks
    ///   for `waves` and `waving` as well.
    /// - Zulip's default search implementation ignores a list of very common
    ///   words like `the` and `a`. See the full list here: TODO: <i literally
    ///   can't find this link... was someone lying? lol>
    /// - Emojis are counted when *used* in messages, though reactions are not
    ///   into account.
    Keyword(String),
    /// The channel a message appears in.
    Channel(NameOrId),
    /// Search within a channel, only including results from a specific topic.
    ChannelWithTopic { channel: NameOrId, topic: NameOrId },
    /// Search direct messages with a given person (or group of people).
    ///
    /// Note that you may include multiple people. This doesn't search in
    /// one-on-one direct messages, but instead, the group direct messages
    /// that ONLY the given people are in.
    DirectMessage(OneOrMany<NameOrId>),
    /// Search direct message chats that include the given person (or people),
    /// alongside any number of other people.
    DirectMessageIncluding(OneOrMany<NameOrId>),
    /// Search in channels with the given channel attribute.
    ///
    /// ## Attributes
    ///
    /// Here's a list of the possible attributes. (right now, there's only one):
    /// - `Public`: looks for messages in all public channels
    Channels(ChannelAttribute),
    /// Looks for messages by a given sender.
    ///
    /// Note that the `MessageSender::Me` variant indicates that the current
    /// user sent the message.
    Sender(MessageSender),
    /// Looks for messages with any attachment, link, or reaction.
    Has(MessageMediaKind),
    /// Finds messages that have the given status.
    Is(MessageStatusKind),
}

/// An enumeration representing the fact that many NarrowKinds take in both
/// object names (e.g. a named stream) or object IDs (e.g. msg_id = `65`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum NameOrId {
    Name(String),
    Id(u64),
}

/// Some NarrowKinds can take one or more parameters. This structure avoids
/// allocating a vector each time you make one of these kinds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

/// An input for the `NarrowKind::Channels` variant. This seems like it may
/// grow in the future based on how it's placed in the API, so here's an enum.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ChannelAttribute {
    /// Any channel for which anyone has access. Including you.
    Public,
    // Cool,
    // Fun,
    // Attractive,
}

/// An input for the `NarrowKind::Sender` variant.
///
/// `Other` represents another person, while `Me` represents the current user.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum MessageSender {
    Other(NameOrId),
    Me,
}

/// An input for the `NarrowKind::Has` variant, representing the various kinds
/// of multimedia a message can contain.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum MessageMediaKind {
    /// The message contains a URL.
    Link,
    /// The message contains an uploaded file (of any kind).
    Attachment,
    /// Contains uploaded or linked images or videos.
    ///
    /// Yes, you read that right - "image" also contains "videos" - be careful
    /// with that!
    Image,
    /// Someone reacted to the message. This feels unique from the other
    /// variants...
    Reaction,
}

/// An input for the `NarrowKind::Is` variant, representing the various
/// statuses a message may have.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum MessageStatusKind {
    /// Contains an [alert word](https://zulip.com/help/dm-mention-alert-notifications#alert-words)
    /// that the user set to cause notifications/"alerts".
    Alerted,
    /// Messages where the user was mentioned.
    Mentioned,
    /// The user starred this message.
    Starred,
    /// Someone posted this message into a topic the user subscribed to!
    Followed,
    /// This message exists in a topic that is resolved.
    Resolved,
    /// The user hasn't yet read this message.
    Unread,
}
