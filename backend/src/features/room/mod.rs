mod websocket;

use crate::state::SharedState;
use axum::routing::any;
use serde::Deserialize;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new().route("/ws", any(websocket::upgrade_handler))
}

#[derive(Deserialize)]
struct WebsocketOptions {
    room: String,
    token: String,
}
