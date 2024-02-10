use ruma_common::serde::Raw;
use ruma_events::{
    exports::serde_json,
    room::message::{deserialize_relation, FormattedBody, Relation},
    AnyMessageLikeEventContent, Mentions, MessageLikeEventType,
};
use ruma_macros::EventContent;
use serde::{de, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum AnyBoardLikeEventContent {
    Post(BoardPostEventContent),
    Reply(BoardReplyEventContent),
}

impl TryFrom<(MessageLikeEventType, Raw<AnyMessageLikeEventContent>)> for AnyBoardLikeEventContent {
    type Error = serde_json::Error;

    fn try_from(
        (event_type, content): (MessageLikeEventType, Raw<AnyMessageLikeEventContent>),
    ) -> Result<Self, Self::Error> {
        let event_type = event_type.to_string();

        match event_type.as_ref() {
            "space.board.post" => Ok(AnyBoardLikeEventContent::Post(content.deserialize_as()?)),
            "space.board.reply" => Ok(AnyBoardLikeEventContent::Reply(content.deserialize_as()?)),
            _ => Err(de::Error::custom("You provided an unknown event type")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "space.board.post", kind = MessageLike, without_relation)]
pub struct BoardPostEventContent {
    /// The title of the post.
    pub title: Option<String>,

    /// The body of the post.
    pub body: String,

    /// Formatted form of the post `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Information about [related posts].
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_relation"
    )]
    pub relates_to: Option<Relation<BoardPostEventContentWithoutRelation>>,

    /// The mentions of this post.
    #[serde(rename = "m.mentions", skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,
}

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "space.board.reply", kind = MessageLike, without_relation)]
pub struct BoardReplyEventContent {
    /// The body of the reply.
    pub body: String,

    /// Formatted form of the reply `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Information about [related replies].
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_relation"
    )]
    pub relates_to: Option<Relation<BoardReplyEventContentWithoutRelation>>,

    /// The mentions of this reply.
    #[serde(rename = "m.mentions", skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Vote {
    Up,
    Down,
}

impl TryInto<String> for Vote {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
    }
}

impl BoardPostEventContent {
    /// A convenience constructor to create a plain text post.
    pub fn plain(body: impl Into<String>) -> Self {
        let body: String = body.into();

        Self {
            title: None,
            body,
            formatted: None,
            relates_to: None,
            mentions: None,
        }
    }

    /// A convenience constructor to create an HTML post.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        let body: String = body.into();
        let formatted = Some(FormattedBody::html(html_body.into()));

        Self {
            title: None,
            body,
            formatted,
            relates_to: None,
            mentions: None,
        }
    }

    /// A convenience constructor to create a Markdown post.
    ///
    /// Returns an HTML post if some Markdown formatting was detected, otherwise
    /// returns a plain text post.
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        if let Some(formatted) = FormattedBody::markdown(&body) {
            Self::html(body, formatted.body)
        } else {
            Self::plain(body)
        }
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }
}

impl BoardReplyEventContent {
    /// A convenience constructor to create a plain text post.
    pub fn plain(body: impl Into<String>) -> Self {
        let body: String = body.into();

        Self {
            body,
            formatted: None,
            relates_to: None,
            mentions: None,
        }
    }

    /// A convenience constructor to create an HTML post.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        let body: String = body.into();
        let formatted = Some(FormattedBody::html(html_body.into()));

        Self {
            body,
            formatted,
            relates_to: None,
            mentions: None,
        }
    }

    /// A convenience constructor to create a Markdown post.
    ///
    /// Returns an HTML post if some Markdown formatting was detected, otherwise
    /// returns a plain text post.
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        if let Some(formatted) = FormattedBody::markdown(&body) {
            Self::html(body, formatted.body)
        } else {
            Self::plain(body)
        }
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::exports::serde_json::{from_value, json};

    use crate::space::board::{BoardPostEventContent, BoardReplyEventContent};

    #[test]
    fn post_deserialize() {
        let json = json!({
              "title": "hi",
              "body": "Rust rewrite coming soon!",
              "format": "org.matrix.custom.html",
              "formatted_body": "<p>Rust rewrite coming soon!</p>\n",
        });

        assert_matches!(
            from_value::<BoardPostEventContent>(json),
            Ok(BoardPostEventContent {
                title,
                body,
                formatted,
                relates_to,
                mentions
            })
        );
        dbg!(&title, &body, &formatted, &relates_to, &mentions);
        assert_eq!(title, Some("hi".to_owned()));
        assert_eq!(body, "Rust rewrite coming soon!");
        assert_eq!(
            formatted.map(|f| f.body),
            Some("<p>Rust rewrite coming soon!</p>\n".to_owned())
        );

        assert!(relates_to.is_none());
        assert!(mentions.is_none());
    }

    #[test]
    fn post_deserialize_err() {
        let json = json!({
              "title": "We forgot the body!",
              "format": "org.matrix.custom.html",
              "formatted_body": "<p>We forgot the body!</p>\n",
        });

        assert_matches!(from_value::<BoardPostEventContent>(json), Err(_Error));
    }

    #[test]
    fn reply_deserialize() {
        let json = json!({
              "body": "Sounds unsafe to me!",
              "format": "org.matrix.custom.html",
              "formatted_body": "<p>Sounds unsafe to me!</p>\n",
        });

        assert_matches!(
            from_value::<BoardReplyEventContent>(json),
            Ok(BoardReplyEventContent {
                body,
                formatted,
                relates_to,
                mentions
            })
        );

        assert_eq!(body, "Sounds unsafe to me!");
        assert_eq!(
            formatted.map(|f| f.body),
            Some("<p>Sounds unsafe to me!</p>\n".to_owned())
        );

        assert!(relates_to.is_none());
        assert!(mentions.is_none());
    }

    #[test]
    fn reply_deserialize_err() {
        let json = json!({
              "format": "org.matrix.custom.html",
              "formatted_body": "<p>Sounds unsafe to me!</p>\n",
        });

        assert_matches!(from_value::<BoardReplyEventContent>(json), Err(_Error));
    }
}
