use crate::state::SharedState;
use axum::routing::post;

mod errors;
mod handlers;
mod models;
mod queries;
mod schemas;
mod service;
mod utils;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new().route("/join-room", post(handlers::join_room))
}
