use axum::Json;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug)]
pub struct AppError {
    pub code: &'static str,     // business error code
    pub message: String,        // what you want the client to see
    pub status: StatusCode,     // HTTP status
    pub details: Option<Value>, // optional details
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let body = Json(ErrorResponse {
            code: self.code.to_string(),
            message: self.message,
            details: self.details,
        });

        (self.status, body).into_response()
    }
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code,
            message: message.into(),
            status,
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = Some(details);
        self
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // log error
        tracing::error!(err=?err, "An unknown internal server error occurred.");

        AppError::new(
            "UNKNOWN_ERROR",
            "An internal server error occurred. Please try again later.",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

const DATABASE_ERROR: &str = "DATABASE_ERROR";
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = err {
            AppError::new(
                DATABASE_ERROR,
                "The requested resource was not found.",
                StatusCode::NOT_FOUND,
            )
        } else {
            tracing::error!(err=?err, "An unknown database error occurred.");
            AppError::new(
                DATABASE_ERROR,
                "An internal database error occurred. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let details = serde_json::to_value(&err).unwrap_or_default();
        AppError::new(
            "VALIDATION_ERROR",
            "The request data is invalid.",
            StatusCode::BAD_REQUEST,
        )
        .with_details(details)
    }
}
