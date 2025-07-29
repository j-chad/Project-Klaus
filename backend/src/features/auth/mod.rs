use crate::state::SharedState;
use axum::routing::post;

mod errors;
mod handlers;
mod middleware;
mod models;
mod queries;
pub(crate) mod schemas;
pub(crate) mod service;
pub(crate) mod utils;

pub use middleware::Session;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/logout", post(handlers::logout))
        .route("/challenge", post(handlers::create_challenge))
        .route("/challenge/verify", post(handlers::verify_challenge))
        .route("/ephemeral", post(handlers::create_ephemeral_token))
}
