#[cfg(test)]
mod tests {
    use std::iter;

    use futures::{future, TryFutureExt};
    use matrix::{
        client::resources::events::{EventsService, GetMessagesQuery},
        filter::RoomEventFilter,
        ruma_common::TransactionId,
        ruma_events::{
            room::message::{
                AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
            },
            MessageLikeEventType,
        },
    };
    use tokio::sync::OnceCell;

    use crate::matrix::util::{self, join_helper, Test};

    static TEST: OnceCell<Test> = OnceCell::const_new();

    #[tokio::test]
    async fn send_message() {
        let Test { admin, samples, .. } = TEST.get_or_init(util::init).await;
        let sample = samples.get(0).unwrap();
        let (owner_id, owner_token) = sample.owner();

        let mut client = admin.clone();
        client.clear_token();

        // first join
        let joins = join_helper(&client, sample.guests(), &sample.room_id).await;

        assert!(joins.iter().all(Result::is_ok));

        future::try_join_all(
            sample
                .guests()
                .map(|(user_id, access_token)| {
                    EventsService::send_message(
                        &client,
                        access_token,
                        &sample.room_id,
                        TransactionId::new(),
                        RoomMessageEventContent::text_markdown(format!(
                            "hello, **my name is {}**",
                            user_id
                        )),
                    )
                })
                .chain(iter::once(EventsService::send_message(
                    &client,
                    owner_token,
                    &sample.room_id,
                    TransactionId::new(),
                    RoomMessageEventContent::text_plain(format!(
                        "and I am the admin of the room, {}",
                        owner_id
                    )),
                ))),
        )
        .await
        .unwrap();

        let expected: Vec<_> = sample
            .guests()
            .map(|(user_id, _)| format!("hello, **my name is {}**", user_id))
            .chain(iter::once(format!(
                "and I am the admin of the room, {}",
                owner_id
            )))
            .collect();

        let filter = RoomEventFilter {
            types: vec![MessageLikeEventType::RoomMessage.into()],
            ..Default::default()
        };

        let filter = serde_json::to_string(&filter).unwrap();

        let found = EventsService::get_messages(
            &client,
            owner_token,
            &sample.room_id,
            GetMessagesQuery {
                limit: Some(111),
                filter: filter.clone(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let found: Vec<_> = found
            .chunk
            .into_iter()
            .map(|e| e.deserialize_as::<OriginalRoomMessageEvent>().unwrap())
            .map(|e| e.content.body().to_owned())
            .collect();

        dbg!(&found, &expected);

        assert!(expected.iter().all(|s| found.contains(s)));
    }

    // TODO
    // #[tokio::test]
    async fn reply_to_message() {
        let Test { admin, samples, .. } = TEST.get_or_init(util::init).await;
        let sample = samples.get(0).unwrap();
        let (owner_id, owner_token) = sample.owner();

        let mut client = admin.clone();
        client.clear_token();

        // first join
        let joins = join_helper(&client, sample.guests(), &sample.room_id).await;
        assert!(joins.iter().all(Result::is_ok));

        let root = EventsService::send_message(
            &client,
            owner_token,
            &sample.room_id,
            TransactionId::new(),
            RoomMessageEventContent::text_plain(format!(
                "I am at the root of the tree, {}",
                owner_id
            )),
        )
        .map_ok(|resp| resp.event_id)
        .await
        .unwrap();

        let n = 5;

        let mut history = Vec::from([vec![root]]);

        for i in 1..n {
            let (user_id, access_token) =
                sample.guests().nth(i % sample.user_ids.len() - 1).unwrap();

            let prev = history.last().unwrap();
            let traverse = future::try_join_all((0..prev.len() * 2).map(|j| {
                EventsService::get_event(
                    &client,
                    access_token,
                    &sample.room_id,
                    prev.get(j / 2).unwrap(),
                )
                .map_ok(|resp| resp.deserialize_as::<OriginalRoomMessageEvent>().unwrap())
                .and_then(|event| {
                    EventsService::send_message(
                        &client,
                        access_token,
                        &sample.room_id,
                        TransactionId::new(),
                        RoomMessageEventContent::text_markdown(format!("level {i}",))
                            .make_reply_to(&event, ForwardThread::No, AddMentions::Yes),
                    )
                })
                .map_ok(|resp| resp.event_id)
            }))
            .await
            .unwrap();

            history.push(traverse.clone());

            tracing::info!(?traverse);
        }

        let filter = RoomEventFilter {
            types: vec![MessageLikeEventType::RoomMessage.into()],
            ..Default::default()
        };

        let filter = serde_json::to_string(&filter).unwrap();

        let found = EventsService::get_messages(
            &client,
            owner_token,
            &sample.room_id,
            GetMessagesQuery {
                limit: Some(111),
                filter: filter.clone(),
                ..Default::default()
            },
        )
        .map_ok(move |resp| resp.chunk)
        .await
        .unwrap();

        assert!(history
            .windows(2)
            .all(|events| events[0].len() * 2 == events[1].len()));
    }
}