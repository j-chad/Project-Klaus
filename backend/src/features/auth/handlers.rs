use super::schemas::{JoinRoomRequest, JoinRoomResponse};
use axum::Json;
use axum::http::StatusCode;

pub async fn join_room(
    Json(payload): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, String)> {
    Err((
        StatusCode::OK,
        "Join room handler not implemented yet".to_string(),
    ))
}
