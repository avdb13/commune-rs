use http::StatusCode;
use matrix::{events::relation::RelationType, ruma_common::OwnedRoomId};
use thiserror::Error;

use crate::error::HttpStatusCode;

#[derive(Debug, Error)]
pub enum BoardErrorCode {
    #[error("You are not a member of the board")]
    NoMembership(OwnedRoomId),
    #[error("You provided an empty or incorrect relation")]
    WrongRelation(RelationType),
}

impl HttpStatusCode for BoardErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            BoardErrorCode::WrongRelation(_) => StatusCode::BAD_REQUEST,
            BoardErrorCode::NoMembership(_) => StatusCode::FORBIDDEN,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            BoardErrorCode::WrongRelation(_) => "BAD_REQUEST",
            BoardErrorCode::NoMembership(_) => "FORBIDDEN",
        }
    }
}
