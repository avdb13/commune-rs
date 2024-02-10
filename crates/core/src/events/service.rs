use std::sync::Arc;

use matrix::{
    client::resources::events::{Events, RelationsParams, SendEventResponse},
    events::{
        reaction::ReactionEventContent,
        relation::{Annotation},
        space::board::{
            BoardPostEvent, BoardPostEventContent, BoardReplyEvent, BoardReplyEventContent, Vote,
        },
        StateEventContent,
    },
    ruma_common::{serde::Raw, EventId, RoomId, TransactionId},
    Client as MatrixAdminClient,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{events::error::BoardErrorCode, util::secret::Secret, Error, Result};

pub struct EventsService {
    admin: Arc<MatrixAdminClient>,
}

#[derive(Serialize)]
pub struct GetPostResponse {
  pub thread: Raw<BoardPostEvent>,
  pub replies: Vec<Raw<BoardReplyEvent>>,
}

impl EventsService {
    pub fn new(admin: Arc<MatrixAdminClient>) -> Self {
        Self { admin }
    }

    pub async fn send_post(
        &self,
        access_token: Secret,
        content: BoardPostEventContent,
        board_id: &RoomId,
        txn_id: &TransactionId,
    ) -> Result<SendEventResponse> {
        let data: SendEventResponse = Events::send_message(
            &self.admin,
            access_token.to_string(),
            content,
            board_id,
            txn_id,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to create new post");
            Error::Unknown
        })?;

        Ok(data)
    }

    pub async fn send_reply(
        &self,
        access_token: Secret,
        content: BoardReplyEventContent,
        board_id: &RoomId,
        txn_id: &TransactionId,
    ) -> Result<SendEventResponse> {
        if let Some(Some(relation)) = content
            .relates_to
            .clone()
            .map(|relation| relation.rel_type())
        {
            return Err(BoardErrorCode::WrongRelation(relation).into());
        };

        let data: SendEventResponse = Events::send_message(
            &self.admin,
            access_token.to_string(),
            content,
            board_id,
            txn_id,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to create new reply");
            Error::Unknown
        })?;

        Ok(data)
    }

    pub async fn send_vote(
        &self,
        access_token: Secret,
        board_id: &RoomId,
        event_id: &EventId,
        txn_id: &TransactionId,
        vote: Vote,
    ) -> Result<SendEventResponse> {
        let vote = vote.try_into().map_err(|err| {
            tracing::error!(?err, "Failed to serialize vote");
            Error::Unknown
        })?;

        let content = ReactionEventContent::new(Annotation::new(event_id.into(), vote));

        let data: SendEventResponse = Events::send_message(
            &self.admin,
            access_token.to_string(),
            content,
            board_id,
            txn_id,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to add new vote");
            Error::Unknown
        })?;

        Ok(data)
    }

    pub async fn send_redaction(
        &self,
        access_token: Secret,
        board_id: &RoomId,
        event_id: &EventId,
        txn_id: &TransactionId,
        reason: Option<String>,
    ) -> Result<SendEventResponse> {
        let data = Events::send_redaction(
            &self.admin,
            access_token.to_string(),
            board_id,
            event_id,
            txn_id,
            reason,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to redact event");
            Error::Unknown
        })?;

        Ok(data)
    }

    pub async fn send_state<C: StateEventContent + DeserializeOwned>(
        &self,
        access_token: Secret,
        content: C,
        board_id: &RoomId,
        state_key: impl AsRef<str>,
    ) -> Result<SendEventResponse> {
        let data = Events::send_state(
            &self.admin,
            access_token.to_string(),
            content,
            board_id,
            state_key,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to create state event");
            Error::Unknown
        })?;

        Ok(data)
    }

    pub async fn get_thread(
        &self,
        access_token: Secret,
        board_id: &RoomId,
        event_id: &EventId,
    ) -> Result<Raw<BoardPostEvent>> {
        let post = Events::get_one(
            &self.admin,
            access_token.to_string(),
            board_id,
            event_id,
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to get post");
            Error::Unknown
        })?;

        Ok(post)
    }

    pub async fn get_replies(
        &self,
        access_token: Secret,
        board_id: &RoomId,
        event_id: &EventId,
        limit: u64,
    ) -> Result<Vec<Raw<BoardReplyEvent>>> {
        let resp = Events::get_relations(
            &self.admin,
            access_token.to_string(),
            board_id,
            event_id,
            Some(None),
            Some("board.post.reply".into()),
            RelationsParams {
                limit: Some(limit),
                ..Default::default()
            },
        )
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to get post");
            Error::Unknown
        })?;

        Ok(resp.chunk)
    }

    pub async fn get_post(
        &self,
        access_token: Secret,
        board_id: &RoomId,
        event_id: &EventId,
    ) -> Result<GetPostResponse> {
      let thread = self.get_thread(access_token.clone(), board_id, event_id).await?;
      let replies = self.get_replies(access_token.clone(), board_id, event_id, 10).await?;

      Ok(GetPostResponse { thread, replies })
    }
}
