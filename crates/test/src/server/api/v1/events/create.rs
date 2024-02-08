use commune::events::service::SendResponse;
use commune_server::router::api::v1::account::login::{AccountLoginPayload, AccountLoginResponse};
use commune_server::router::api::v1::events::{CreatePostDto};
use matrix::events::space::board::BoardPostEventContent;

use crate::tools::http::HttpClient;

#[tokio::test]
async fn creates_post() {
    let http_client = HttpClient::new().await;

    let username: String = "steve".to_owned();
    let password: String = "verysecure".to_owned();

    let response = http_client
        .post("/api/v1/account/login")
        .json(&AccountLoginPayload { username, password })
        .send()
        .await;

    let response_payload = response.json::<AccountLoginResponse>().await;

    let payload = CreatePostDto {
        board_id: "!qanGtaocRIWXMEYshH:matrix.localhost".into(),
        content: BoardPostEventContent::plain("hello world"),
        kind: "post".to_owned(),
        txn_id: None,
    };

    let create_post_res = http_client
        .put("/api/v1/event")
        .json(&payload)
        .token(response_payload.access_token)
        .send()
        .await;
    let json_res: SendResponse = create_post_res.json().await;
    dbg!(json_res);
}
