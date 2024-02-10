use std::sync::Arc;

use tracing::instrument;

use matrix::{
    client::resources::room::{
        CreateRoomBody, JoinRoomBody, Room as MatrixRoom, RoomCreationContent, RoomPreset, ForgetRoomBody, LeaveRoomBody,
    },
    ruma_common::{OwnedUserId, OwnedRoomId, OwnedRoomOrAliasId},
    Client as MatrixAdminClient,
};
use crate::{util::secret::Secret, Error, Result};

use super::model::Room;

#[derive(Debug, Default, Clone)]
pub struct CreateRoomDto {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateDirectRoomDto {
    pub invitee: OwnedUserId,
}

#[derive(Debug, Clone)]
pub struct JoinRoomDto {
    pub alias_or_id: OwnedRoomOrAliasId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArchiveRoomDto {
    pub room_id: OwnedRoomId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LeaveRoomDto {
    pub room_id: OwnedRoomId,
    pub reason: Option<String>,
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
                name: dto.name.unwrap_or_default(),
                room_alias_name: dto.alias.unwrap_or_default(),
                topic: dto.topic.unwrap_or_default(),
                ..Default::default()
            },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to create public room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    /// Creates a Hidden Chat Room
    #[instrument(skip(self, dto))]
    pub async fn create_hidden_room(
        &self,
        access_token: &Secret,
        dto: CreateRoomDto,
    ) -> Result<Room> {
        match MatrixRoom::create(
            &self.admin,
            access_token.to_string(),
            CreateRoomBody {
                creation_content: Some(RoomCreationContent { federate: false }),
                preset: Some(RoomPreset::PrivateChat),
                name: dto.name.unwrap_or_default(),
                room_alias_name: dto.alias.unwrap_or_default(),
                topic: dto.topic.unwrap_or_default(),
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
    #[instrument(skip(self, dto))]
    pub async fn create_direct_room(
        &self,
        access_token: &Secret,
        dto: CreateDirectRoomDto,
    ) -> Result<Room> {
        match MatrixRoom::create(
            &self.admin,
            access_token.to_string(),
            CreateRoomBody {
                creation_content: Some(RoomCreationContent { federate: false }),
                preset: Some(RoomPreset::TrustedPrivateChat),
                is_direct: true,
                invite: vec![dto.invitee],
                ..Default::default()
            },
        )
        .await
        {
            Ok(room) => Ok(Room {
                room_id: room.room_id,
            }),
            Err(err) => {
                tracing::error!("Failed to create direct room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    /// Joins a Chat Room
    #[instrument(skip(self, dto))]
    pub async fn join_room(
        &self,
        access_token: &Secret,
        dto: JoinRoomDto,
    ) -> Result<Room> {
        match MatrixRoom::join(
            &self.admin,
            access_token.to_string(),
            &dto.alias_or_id,
            JoinRoomBody { reason: dto.reason.unwrap_or_default() },
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

    /// Archives a Chat Room
    #[instrument(skip(self, dto))]
    pub async fn archive_room(
        &self,
        access_token: &Secret,
        dto: ArchiveRoomDto,
    ) -> Result<()> {
        match MatrixRoom::forget(
            &self.admin,
            access_token.to_string(),
            &dto.room_id,
            ForgetRoomBody { reason: dto.reason.unwrap_or_default() },
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to archive room: {}", err);
                Err(Error::Unknown)
            }
        }
    }

    /// Leaves a Chat Room
    #[instrument(skip(self, dto))]
    pub async fn leave_room(
        &self,
        access_token: &Secret,
        dto: LeaveRoomDto,
    ) -> Result<()> {
        match MatrixRoom::leave(
            &self.admin,
            access_token.to_string(),
            &dto.room_id,
            LeaveRoomBody { reason: dto.reason.unwrap_or_default() },
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to leave room: {}", err);
                Err(Error::Unknown)
            }
        }
    }
}
