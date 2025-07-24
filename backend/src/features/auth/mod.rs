use crate::state::SharedState;
use axum::routing::post;

mod errors;
mod handlers;
mod middleware;
mod models;
mod queries;
mod schemas;
pub(crate) mod service;
mod utils;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/join-room", post(handlers::join_room))
        .route("/logout", post(handlers::logout))
        .route("/challenge", post(handlers::create_challenge))
        .route("/challenge/verify", post(handlers::verify_challenge))
        .route("/ephemeral", post(handlers::create_ephemeral_token))
}
