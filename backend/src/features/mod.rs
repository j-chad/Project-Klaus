use crate::state::SharedState;
use axum::Router;

// pub mod auth;
mod auth;
mod health;
mod room;
mod utils;

pub fn build_router() -> Router<SharedState> {
    Router::new().nest("/v1", build_v1_router())
}

fn build_v1_router() -> Router<SharedState> {
    Router::new()
        .nest("/health", health::build_router())
        .nest("/room", room::build_router())
        .nest("/auth", auth::build_router())
}
