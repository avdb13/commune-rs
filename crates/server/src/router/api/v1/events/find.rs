use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use commune::events::ruma_common::{OwnedRoomId, OwnedEventId};

use crate::router::api::ApiError;
use crate::router::middleware::AccessToken;
use crate::services::SharedServices;

pub async fn thread(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path((board_id, event_id)): Path<(OwnedRoomId, OwnedEventId)>
) -> Response {
    match services
        .commune
        .events
        .get_post(
            access_token.into(),
            &board_id,
            &event_id,
        )
        .await
    {
        Ok(resp) => {
            let mut response = Json(resp).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => ApiError::from(err).into_response(),
    }
}

pub async fn replies(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path((board_id, event_id)): Path<(OwnedRoomId, OwnedEventId)>
) -> Response {
    match services
        .commune
        .events
        .get_replies(
            access_token.into(),
            &board_id,
            &event_id,
            10
        )
        .await
    {
        Ok(resp) => {
            let mut response = Json(resp).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => ApiError::from(err).into_response(),
    }
}


pub async fn handler(
    Extension(services): Extension<SharedServices>,
    Extension(access_token): Extension<AccessToken>,
    Path((board_id, event_id)): Path<(OwnedRoomId, OwnedEventId)>
) -> Response {
    match services
        .commune
        .events
        .get_post(
            access_token.into(),
            &board_id,
            &event_id,
        )
        .await
    {
        Ok(resp) => {
            let mut response = Json(resp).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => ApiError::from(err).into_response(),
    }
}
