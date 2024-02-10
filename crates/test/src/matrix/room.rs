use commune::account::{model::Account, service::CreateUnverifiedAccountDto};
use fake::{
    faker,
    faker::internet::en::{Password, SafeEmail, Username},
    Fake,
};
use matrix::{
    admin::resources::room::{
        DeleteParams, ListParams, ListResponse, RoomService as AdminRoomService,
    },
    Client,
};

use commune::{room::service::CreateRoomDto, util::secret::Secret};

use crate::tools::environment::Environment;

struct AccountWithRoom {
    account: Account,
    _access_token: String,
    room_dto: CreateRoomDto,
    room_id: String,
}

async fn create_rooms(env: &Environment, i: usize) -> Vec<AccountWithRoom> {
    let mut result = Vec::with_capacity(i);

    for j in 0..i {
        let account_dto = CreateUnverifiedAccountDto {
            username: Username().fake::<String>().chars().take(12).collect(),
            password: Secret::new(Password(10..20).fake::<String>()),
            email: SafeEmail().fake::<String>(),
        };

        let room_dto = CreateRoomDto {
            name: format!("{j} - {username}'s room", username = account_dto.username),
            topic: format!(
                "{j} - discussion about {buzzword}",
                buzzword = faker::company::en::Buzzword().fake::<String>()
            ),
            alias: format!("{j}-{username}", username = account_dto.username),
        };

        let account = env
            .commune
            .account
            .register_unverified(account_dto)
            .await
            .unwrap();
        let access_token = env
            .commune
            .account
            .issue_user_token(account.user_id.clone())
            .await
            .unwrap();
        let resp = env
            .commune
            .room
            .create_public_room(&Secret::new(access_token.clone()), room_dto.clone())
            .await
            .unwrap();

        result.push(AccountWithRoom {
            account,
            _access_token: access_token,
            room_dto,
            room_id: resp.room_id,
        })
    }

    result
}

async fn remove_rooms(client: &Client) {
    let ListResponse { rooms, .. } = AdminRoomService::get_all(&client, ListParams::default())
        .await
        .unwrap();

    for room in rooms {
        AdminRoomService::delete_room(
            &client,
            room.room_id.as_ref(),
            DeleteParams {
                new_room: None,
                block: true,
                purge: true,
            },
        )
        .await
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::future;

    use matrix::{
        admin::resources::room::{EventContextParams, MessagesParams, OrderBy},
        events::{AnyMessageLikeEvent, AnyStateEvent},
        ruma_common::{server_name, EventId, OwnedEventId, RoomId, ServerName},
    };
    use tokio::sync::{futures, OnceCell};

    use super::*;

    static ENVIRONMENT: OnceCell<Environment> = OnceCell::const_new();
    static ACCOUNTS: OnceCell<Vec<AccountWithRoom>> = OnceCell::const_new();
    static RAND_EVENT_ID: OnceCell<OwnedEventId> = OnceCell::const_new();

    async fn init_env() -> Environment {
        let mut env = Environment::new().await;
        env.client
            .set_token(env.config.synapse_admin_token.clone())
            .unwrap();
        remove_rooms(&env.client).await;

        env
    }

    async fn init_accounts() -> Vec<AccountWithRoom> {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = create_rooms(&env, 10).await;

        accounts_with_room
    }

    #[tokio::test]
    async fn get_all_rooms() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let ListResponse { rooms: resp, .. } = AdminRoomService::get_all(
            &env.client,
            ListParams {
                order_by: Some(OrderBy::Name),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(
            accounts_with_room
                .iter()
                .enumerate()
                .map(|(i, acc)| Some(format!("{} - {}'s room", i, acc.account.display_name)))
                .collect::<Vec<_>>(),
            resp.iter().map(|r| r.name.clone()).collect::<Vec<_>>(),
        );
        assert_eq!(
            accounts_with_room
                .iter()
                .map(|acc| format!("#{}:{}", acc.room_dto.alias, env.config.synapse_server_name))
                .collect::<Vec<_>>(),
            resp.iter()
                .map(|r| r.canonical_alias.clone().unwrap())
                .collect::<Vec<_>>(),
        );

        let ListResponse { rooms: resp, .. } = AdminRoomService::get_all(
            &env.client,
            ListParams {
                order_by: Some(OrderBy::Creator),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let mut creators = accounts_with_room
            .iter()
            .map(|acc| acc.account.username.clone())
            .collect::<Vec<_>>();
        creators.sort();

        assert_eq!(
            creators,
            resp.iter()
                .map(|r| r.creator.clone().unwrap())
                .collect::<Vec<_>>(),
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn get_all_rooms_err() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let ListResponse { rooms: resp, .. } = AdminRoomService::get_all(
            &env.client,
            ListParams {
                from: Some(u64::MAX),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_room_details() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let magic_number = Box::into_raw(Box::new(12345)) as usize % accounts_with_room.len();
        let rand = accounts_with_room.iter().nth(magic_number).unwrap();

        let resp =
            AdminRoomService::get_one(&env.client, &RoomId::parse(rand.room_id.clone()).unwrap())
                .await
                .unwrap();

        assert_eq!(
            Some(format!(
                "#{}:{}",
                rand.room_dto.alias, env.config.synapse_server_name
            )),
            resp.canonical_alias,
        );
        assert_eq!(Some(rand.room_dto.name.clone()), resp.name);
        assert_eq!(Some(rand.account.username.clone()), resp.creator,);
        assert_eq!(
            Some(rand.room_dto.topic.clone()),
            resp.details.map(|d| d.topic).flatten(),
        );
        assert_eq!(resp.join_rules, Some("public".into()));

        assert!(!resp.public);
        assert!(resp.room_type.is_none());
    }

    #[tokio::test]
    #[should_panic]
    async fn get_room_details_err() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let resp = AdminRoomService::get_one(
            &env.client,
            &RoomId::new(<&ServerName>::try_from(env.config.synapse_server_name.as_str()).unwrap()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_room_events() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let magic_number = Box::into_raw(Box::new(12345)) as usize % accounts_with_room.len();
        let rand = accounts_with_room.iter().nth(magic_number).unwrap();

        let resp = AdminRoomService::get_room_events::<AnyStateEvent>(
            &env.client,
            &RoomId::parse(rand.room_id.clone()).unwrap(),
            // no idea what the type is
            MessagesParams {
                from: "".into(),
                to: None,
                limit: None,
                filter: None,
                direction: None,
            },
        )
        .await
        .unwrap();

        let events = resp.chunk.deserialize().unwrap();
        let rand_event = events.get(magic_number % events.len()).unwrap();

        RAND_EVENT_ID
            .set(rand_event.clone().event_id().to_owned())
            .unwrap();

        assert!(events.len() == 8,);
    }

    #[tokio::test]
    #[should_panic]
    async fn get_room_events_err() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let resp = AdminRoomService::get_room_events::<AnyStateEvent>(
            &env.client,
            &RoomId::new(<&ServerName>::try_from(env.config.synapse_server_name.as_str()).unwrap()),
            MessagesParams {
                from: "".into(),
                to: None,
                limit: None,
                filter: None,
                direction: None,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_state_events() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let magic_number = Box::into_raw(Box::new(12345)) as usize % accounts_with_room.len();
        let rand = accounts_with_room.iter().nth(magic_number).unwrap();

        let resp =
            AdminRoomService::get_state(&env.client, &RoomId::parse(rand.room_id.clone()).unwrap())
                .await
                .unwrap();

        assert!(resp
            .state
            .into_iter()
            .all(|state| state.kind.contains("room")));
    }

    #[tokio::test]
    #[should_panic]
    async fn get_state_events_err() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let resp = AdminRoomService::get_state(
            &env.client,
            &RoomId::new(<&ServerName>::try_from(env.config.synapse_server_name.as_str()).unwrap()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_members() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let magic_number = Box::into_raw(Box::new(12345)) as usize % accounts_with_room.len();
        let rand = accounts_with_room.iter().nth(magic_number).unwrap();

        let resp = AdminRoomService::get_members(
            &env.client,
            &RoomId::parse(rand.room_id.clone()).unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(resp.members, vec![rand.account.username.clone()]);
    }

    #[tokio::test]
    #[should_panic]
    async fn get_members_err() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        let resp = AdminRoomService::get_members(
            &env.client,
            &RoomId::new(<&ServerName>::try_from(env.config.synapse_server_name.as_str()).unwrap()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_event_context() {
        let env = ENVIRONMENT.get_or_init(init_env).await;
        let accounts_with_room = ACCOUNTS.get_or_init(init_accounts).await;

        while let false = !RAND_EVENT_ID.initialized() {}

        let magic_number = Box::into_raw(Box::new(12345)) as usize % accounts_with_room.len();
        let rand = accounts_with_room.iter().nth(magic_number).unwrap();

        let _resp = AdminRoomService::get_event_context(
            &env.client,
            &RoomId::parse(rand.room_id.clone()).unwrap(),
            &RAND_EVENT_ID.get().unwrap(),
            EventContextParams::default(),
        )
        .await
        .unwrap();
    }
}
