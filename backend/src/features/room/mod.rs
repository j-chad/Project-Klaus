mod errors;
mod handlers;
mod models;
mod queries;
mod schemas;
mod service;
mod websocket;

use crate::state::SharedState;
use axum::routing::{any, post};
use serde::Deserialize;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/ws", any(websocket::upgrade_handler))
        .route("/create", post(handlers::create_room))
        .route("/join", post(handlers::join_room))
        .route("/start", post(handlers::start_game))
        .route("/publish/message", post(handlers::handle_santa_id_message))
        .route("/commit/seed", post(handlers::commit_seed))
}

#[derive(Deserialize)]
struct WebsocketOptions {
    room: String,
    token: String,
}
