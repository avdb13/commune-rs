use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use commune::events::ruma_common::serde::Raw;
use commune::events::ruma_common::{OwnedRoomId, OwnedTransactionId};
use commune::events::space::board::AnyBoardLikeEventContent;
use commune::events::{AnyMessageLikeEventContent, MessageLikeEventType};
use serde_json::json;

use crate::router::api::ApiError;
use crate::router::middleware::AccessToken;
use crate::services::SharedServices;

pub async fn handler(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path((board_id, event_type, txn_id)): Path<(
        OwnedRoomId,
        MessageLikeEventType,
        OwnedTransactionId,
    )>,
    Json(content): Json<Raw<AnyMessageLikeEventContent>>,
) -> Response {
    let content = match AnyBoardLikeEventContent::try_from((event_type, content)) {
        Ok(content) => content,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let request = match content {
        AnyBoardLikeEventContent::Post(content) => {
            services
                .commune
                .events
                .send_post(access_token.into(), content, &board_id, &txn_id)
                .await
        }
        AnyBoardLikeEventContent::Reply(content) => {
            services
                .commune
                .events
                .send_reply(access_token.into(), content, &board_id, &txn_id)
                .await
        }
    };

    match request {
        Ok(resp) => {
            let mut response = Json(json!({"event_id": resp.0})).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => ApiError::from(err).into_response(),
    }
}
