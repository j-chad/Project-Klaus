use crate::error::AppError;
use axum::http::StatusCode;

#[derive(Debug)]
pub enum AuthError {
    RoomNotFound,
    InvalidPublicKey,
    RoomFull,
    TokenGenerationFailed,
    ExpiredToken,
    MissingToken,
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::RoomNotFound => AppError::new(
                "ROOM_NOT_FOUND",
                "The specified room does not exist.",
                StatusCode::NOT_FOUND,
            ),
            AuthError::InvalidPublicKey => AppError::new(
                "INVALID_PUBLIC_KEY",
                "The provided public key is invalid or malformed.",
                StatusCode::BAD_REQUEST,
            ),
            AuthError::RoomFull => AppError::new(
                "ROOM_FULL",
                "The room is full. Please try another room.",
                StatusCode::FORBIDDEN,
            ),
            AuthError::TokenGenerationFailed => AppError::new(
                "TOKEN_GENERATION_FAILED",
                "Failed to generate a secure token. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            AuthError::ExpiredToken => AppError::new(
                "EXPIRED_TOKEN",
                "The provided token has expired. Please log in again.",
                StatusCode::UNAUTHORIZED,
            ),
            AuthError::MissingToken => AppError::new(
                "MISSING_TOKEN",
                "No authentication token provided. Please log in.",
                StatusCode::UNAUTHORIZED,
            ),
        }
    }
}
