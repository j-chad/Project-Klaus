use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;

mod service;

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/ping", post(|| async { "pong" }))
        .route("/check", post(health_check))
}

async fn health_check(State(state): State<SharedState>) -> impl IntoResponse {
    let report = service::run_health_checks(&state).await;

    let status = match report.result {
        service::HealthCheckResult::Ok => StatusCode::OK,
        service::HealthCheckResult::Failed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, Json(report))
}