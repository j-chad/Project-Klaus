use super::queries;
use super::schemas::{JoinRoomRequest, JoinRoomResponse};
use crate::error::AppError;
use crate::state::SharedState;
use axum::Json;
use axum::extract::State;

pub async fn join_room(
    State(state): State<SharedState>,
    Json(body): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, AppError> {
    let user = service::register_user(&state.db, &body);
    let room = queries::get_room_by_join_code(&state.db, &payload.room_id).await?;

    if let Some(max) = room.max_members {
        let current_members = queries::get_current_member_count(&state.db, room.id).await?;
        if current_members >= max {
            return Err(AppError::new(
                "ROOM_FULL",
                "The room is full. Please try another room.",
                axum::http::StatusCode::FORBIDDEN,
            ));
        }
    }
}
