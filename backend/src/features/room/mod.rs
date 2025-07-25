mod handlers;
mod schemas;
mod websocket;

use crate::state::SharedState;
use axum::routing::{any, post};
use serde::Deserialize;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/ws", any(websocket::upgrade_handler))
        .route("/create-room", post(handlers::create_room))
        .route("/join-room", post(handlers::join_room))
}

#[derive(Deserialize)]
struct WebsocketOptions {
    room: String,
    token: String,
}
