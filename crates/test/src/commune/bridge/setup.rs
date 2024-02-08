use commune::{util::secret::Secret, room::service::CreateRoomDto, account::{service::CreateUnverifiedAccountDto, model::Account}};
use fake::{faker::{internet::en::{Username, Password, SafeEmail}, company::en::Buzzword}, Fake};
use matrix::ruma_common::UserId;

use crate::tools::environment::Environment;

#[tokio::test]
async fn ok() {
    let env = Environment::new().await;

    let (account, access_token) = create_user(&env).await;
    let invitee = &UserId::parse(format!("@discordbot:{server_name}", server_name = env.config.synapse_server_name)).unwrap();

    env.commune.room.create_direct_room(&Secret::new(access_token), invitee);
}

async fn create_user(env: &Environment) -> (Account, String) {
        let account_dto = CreateUnverifiedAccountDto {
            username: Username().fake::<String>().chars().take(12).collect(),
            password: Secret::new(Password(10..20).fake::<String>()),
            email: SafeEmail().fake::<String>(),
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

        (account, access_token)
}
