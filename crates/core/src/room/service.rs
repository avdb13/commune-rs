use std::sync::Arc;

use tracing::instrument;

use matrix::{
    client::resources::room::{
        CreateRoomBody, JoinRoomBody, Room as MatrixRoom, RoomCreationContent, RoomPreset,
    },
    ruma_common::{RoomId, UserId},
    Client as MatrixAdminClient,
};

use crate::{util::secret::Secret, Error, Result};

use super::model::Room;

#[derive(Debug, Clone)]
pub struct CreateRoomDto {
    pub name: String,
    pub topic: String,
    pub alias: String,
}

pub struct RoomService {
    admin: Arc<MatrixAdminClient>,
}

impl RoomService {
    pub fn new(admin: Arc<MatrixAdminClient>) -> Self {
        Self { admin }
    }

    /// Creates a Public Chat Room
    #[instrument(skip(self, dto))]
    pub async fn create_public_room(
        &self,
        access_token: &Secret,
        dto: CreateRoomDto,
    ) -> Result<Room> {
        match MatrixRoom::create(
            &self.admin,
            access_token.to_string(),
            CreateRoomBody {
                creation_content: Some(RoomCreationContent { federate: false }),
                preset: Some(RoomPreset::PublicChat),
                name: dto.name,
                room_alias_name: dto.alias,
                topic: dto.topic,
                ..Default::default()
            },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to create room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    /// Creates a Direct Chat Room
    #[instrument(skip(self))]
    pub async fn create_trusted_private_room(
        &self,
        access_token: &Secret,
        invitee: &UserId,
    ) -> Result<Room> {
        match MatrixRoom::create(
            &self.admin,
            access_token.to_string(),
            CreateRoomBody {
                creation_content: Some(RoomCreationContent { federate: false }),
                preset: Some(RoomPreset::TrustedPrivateChat),
                is_direct: true,
                invite: vec![invitee.to_owned()],
                ..Default::default()
            },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to create room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn join_room(
        &self,
        access_token: &Secret,
        room_id: &RoomId,
        reason: String,
    ) -> Result<Room> {
        match MatrixRoom::join(
            &self.admin,
            access_token.to_string(),
            room_id.into(),
            JoinRoomBody { reason },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to join room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn forget_room(
        &self,
        access_token: &Secret,
        room_id: &RoomId,
        reason: String,
    ) -> Result<Room> {
        match MatrixRoom::join(
            &self.admin,
            access_token.to_string(),
            room_id.into(),
            JoinRoomBody { reason },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to join room: {}", err);
                Err(Error::Unknown)
            }
        }
    }
}
