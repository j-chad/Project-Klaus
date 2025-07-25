use crate::error::AppError;
use axum::http::StatusCode;

pub enum RoomError {
    RequiresOwnerPermission,
}

impl From<RoomError> for AppError {
    fn from(err: RoomError) -> Self {
        match err {
            RoomError::RequiresOwnerPermission => AppError::new(
                "REQUIRES_OWNER_PERMISSION",
                "This action requires owner permissions.",
                StatusCode::FORBIDDEN,
            ),
        }
    }
}
