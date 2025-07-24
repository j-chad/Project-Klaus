use crate::features::build_router;
use crate::state::SharedState;
use axum::Router;
use tower_http::trace::TraceLayer;

pub fn create_app(state: SharedState) -> Router {
    Router::new()
        .nest("/api", build_router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}