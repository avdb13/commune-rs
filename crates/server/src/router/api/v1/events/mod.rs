pub mod send;
pub mod redact;
pub mod find;

use axum::{Router, routing::{put, get}, middleware};

use crate::router::middleware::auth;

pub struct Events;

impl Events {
    pub fn routes() -> Router {
        let protected = Router::new()
            .route("/send/:event_type/:txn_id", put(send::handler))
            .route("/redact/:event_id/:txn_id", put(redact::handler))
            .route_layer(middleware::from_fn(auth));

        protected.nest(
            "/:event_id",
            Router::new()
                .route("/", get(find::handler))
                .route("/thread", get(find::thread))
                .route("/replies", get(find::replies)),
        )
    }
}
