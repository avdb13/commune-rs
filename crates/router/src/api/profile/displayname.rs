use axum::{
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub displayname: String,
}

pub async fn handler(
    TypedHeader(access_token): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Response {
    use commune::profile::displayname::service;

    match service(access_token.token(), payload.displayname).await {
        Ok(_) => Json(crate::EmptyBody {}).into_response(),
        Err(e) => {
            tracing::warn!(?e, "failed to update displayname");

            e.into_response()
        }
    }
}