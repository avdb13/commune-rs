use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use commune::events::ruma_common::serde::Raw;
use commune::events::space::state::SpaceRestrictionEventContent;
use serde_json::json;

use crate::router::api::ApiError;
use crate::router::middleware::AccessToken;
use crate::services::SharedServices;

pub async fn handler(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path(board_id): Path<String>,
    Path(event_type): Path<String>,
    Path(state_key): Path<String>,
    Json(content): Json<Raw<SpaceRestrictionEventContent>>,
) -> Response {
    match services
        .commune
        .events
        .send_state(
            access_token.into(),
            content,
            board_id,
            state_key,
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
