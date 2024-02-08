use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use commune::events::ruma_common::{OwnedRoomId, OwnedEventId, OwnedTransactionId};
use serde::Deserialize;
use serde_json::json;

use crate::router::api::ApiError;
use crate::router::middleware::AccessToken;
use crate::services::SharedServices;

#[derive(Deserialize)]
pub struct RedactEventDto {
    reason: Option<String>,
}

pub async fn handler(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path((board_id, event_id, txn_id)): Path<(OwnedRoomId, OwnedEventId, OwnedTransactionId)>,
    Json(payload): Json<RedactEventDto>,
) -> Response {
    match services
        .commune
        .events
        .send_redaction(
            access_token.into(),
            &board_id,
            &event_id,
            &txn_id,
            payload.reason,
        )
        .await
    {
        Ok(resp) => {
            let mut response = Json(json!({"event_id": resp.0})).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => ApiError::from(err).into_response(),
    }
}
