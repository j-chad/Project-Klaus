use crate::error::AppError;
use crate::features::room::models::GamePhase;
use axum::http::StatusCode;

pub enum RoomError {
    RoomNotFound,
    RoomFull,
    RequiresOwnerPermission,
    InvalidGamePhase(ExpectedCurrent<GamePhase>),
    AlreadySentMessage,
}

#[derive(Debug, serde::Serialize)]
pub struct ExpectedCurrent<T> {
    pub(crate) expected: T,
    pub(crate) current: T,
}

impl From<RoomError> for AppError {
    fn from(err: RoomError) -> Self {
        match err {
            RoomError::RoomNotFound => AppError::new(
                "ROOM_NOT_FOUND",
                "The specified room does not exist.",
                StatusCode::NOT_FOUND,
            ),
            RoomError::RoomFull => AppError::new(
                "ROOM_FULL",
                "The room is full. Please try another room.",
                StatusCode::FORBIDDEN,
            ),
            RoomError::RequiresOwnerPermission => AppError::new(
                "REQUIRES_OWNER_PERMISSION",
                "This action requires owner permissions.",
                StatusCode::FORBIDDEN,
            ),
            RoomError::InvalidGamePhase(expected_current) => AppError::new(
                "INVALID_GAME_PHASE",
                "The game is not in the correct phase for this action.",
                StatusCode::BAD_REQUEST,
            )
            .with_serializable_details(expected_current),
            RoomError::AlreadySentMessage => AppError::new(
                "ALREADY_SENT_MESSAGE",
                "You have already sent a message in this round.",
                StatusCode::BAD_REQUEST,
            ),
        }
    }
}
