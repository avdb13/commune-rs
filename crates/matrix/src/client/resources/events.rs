use anyhow::Result;
use ruma_common::{EventId, RoomId, TransactionId};

use ruma_events::relation::RelationType;
use ruma_events::room::redaction::RoomRedactionEventContent;
use ruma_events::{MessageLikeEventContent, MessageLikeEventType, StateEventContent};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::admin::resources::room::Direction;
use crate::Client;

pub struct Events;

#[derive(Debug, Default, Serialize)]
pub struct RelationsParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<u64>,
    pub direction: Option<Direction>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct SendEventResponse(pub String);

#[derive(Debug, Deserialize)]
pub struct RelationsResponse<T> {
    pub chunk: Vec<T>,
    pub prev_batch: Option<String>,
    pub next_batch: Option<String>,
}

impl Events {
    #[instrument(skip(client, access_token, content, room_id))]
    pub async fn send_message<T: MessageLikeEventContent>(
        client: &Client,
        access_token: impl Into<String>,
        content: T,
        room_id: &RoomId,
        txn_id: &TransactionId,
    ) -> Result<SendEventResponse> {
        let mut tmp = (*client).clone();
        tmp.set_token(access_token)?;

        let resp = tmp
            .put_json(
                format!(
                    "/_matrix/client/v3/rooms/{room_id}/send/{event_type}/{txn_id}",
                    room_id = room_id,
                    event_type = content.event_type(),
                    txn_id = txn_id,
                ),
                &content,
            )
            .await?;

        Ok(resp.json().await?)
    }

    #[instrument(skip(client, access_token, content, room_id, state_key))]
    pub async fn send_state<T: StateEventContent>(
        client: &Client,
        access_token: impl Into<String>,
        content: T,
        room_id: &RoomId,
        state_key: impl AsRef<str>,
    ) -> Result<SendEventResponse> {
        let mut tmp = (*client).clone();
        tmp.set_token(access_token)?;

        let resp = tmp
            .put_json(
                format!(
                    "/_matrix/client/v3/rooms/{room_id}/state/{event_type}/{state_key}",
                    room_id = room_id,
                    event_type = content.event_type(),
                    state_key = state_key.as_ref(),
                ),
                &content,
            )
            .await?;

        Ok(resp.json().await?)
    }

    #[instrument(skip(client, access_token, room_id, event_id))]
    pub async fn send_redaction(
        client: &Client,
        access_token: impl Into<String>,
        room_id: &RoomId,
        event_id: &EventId,
        txn_id: &TransactionId,
        reason: Option<String>,
    ) -> Result<SendEventResponse> {
        let mut tmp = (*client).clone();
        tmp.set_token(access_token)?;

        let content = RoomRedactionEventContent::new_v11(event_id.into());

        let resp = tmp
            .put_json(
                format!(
                    "/_matrix/client/v3/rooms/{room_id}/redact/{event_id}/{txn_id}",
                    room_id = room_id,
                    event_id = event_id,
                    txn_id = txn_id,
                ),
                &content,
            )
            .await?;

        Ok(resp.json().await?)
    }

    #[instrument(skip(client, access_token, room_id, event_id))]
    pub async fn get_one<M: DeserializeOwned>(
        client: &Client,
        access_token: impl Into<String>,
        room_id: &RoomId,
        event_id: &EventId,
    ) -> Result<M> {
        let mut tmp = (*client).clone();
        tmp.set_token(access_token)?;

        let resp = tmp
            .get(format!(
                "/_matrix/client/v3/rooms/{room_id}/event/{event_id}",
                room_id = room_id,
                event_id = event_id,
            ))
            .await?;

        Ok(resp.json().await?)
    }

    #[instrument(skip(client, access_token, room_id, event_id))]
    pub async fn get_relations<M: DeserializeOwned>(
        client: &Client,
        access_token: impl Into<String>,
        room_id: &RoomId,
        event_id: &EventId,
        rel_type: Option<Option<RelationType>>,
        event_type: Option<MessageLikeEventType>,
        params: RelationsParams,
    ) -> Result<RelationsResponse<M>> {
        let mut tmp = (*client).clone();
        tmp.set_token(access_token)?;

        let mut path = format!(
            "/_matrix/client/v3/rooms/{room_id}/relations/{event_id}",
            room_id = room_id,
            event_id = event_id,
        );

        if let Some(rel_type) = rel_type {
            path.push_str(&format!(
                "/{rel_type}",
                rel_type = rel_type
                    .map(|rel| rel.to_string())
                    .unwrap_or("m.in_reply_to".into())
            ))
        }

        if let Some(event_type) = event_type {
            path.push_str(&format!("/{event_type}", event_type = event_type))
        }

        let resp = tmp.get_query(path, &params).await?;

        Ok(resp.json().await?)
    }
}
