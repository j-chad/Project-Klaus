use crate::error::AppError;
use axum::http::StatusCode;

#[derive(Debug)]
pub enum AuthError {
    RoomNotFound,
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::RoomNotFound => AppError::new(
                "ROOM_NOT_FOUND",
                "The specified room does not exist.",
                StatusCode::NOT_FOUND,
            ),
        }
    }
}
