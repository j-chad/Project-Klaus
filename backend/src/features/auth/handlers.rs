use axum::http::StatusCode;
use axum::Json;
use super::schemas::{JoinRoomRequest, JoinRoomResponse};

pub async fn join_room(
    Json(payload): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, String)> {
    let room_id = payload.room_id;
    let user_id = payload.user_id;

    // Validate the room ID and user ID
    if room_id.is_empty() || user_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Invalid room or user ID".to_string()));
    }

    // Attempt to join the room
    match state.room_service.join_room(room_id, user_id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
})