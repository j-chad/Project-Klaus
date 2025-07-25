use crate::error::AppError;
use axum::http::StatusCode;

pub enum RoomError {
    RoomNotFound,
    RoomFull,
    RequiresOwnerPermission,
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
        }
    }
}
