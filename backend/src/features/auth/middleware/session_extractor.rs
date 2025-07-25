use super::super::errors::AuthError;
use super::super::models;
use super::super::queries;
use crate::error::AppError;
use crate::state::SharedState;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;

#[derive(Debug)]
pub struct Session(pub models::Token);

fn extract_token(parts: &mut Parts, state: &SharedState) -> Result<String, AppError> {
    // 1. Try cookie first
    let cookies = CookieJar::from_headers(&parts.headers);
    if let Some(cookie) = cookies.get(&state.config.auth.session_cookie_name) {
        return Ok(cookie.value().to_string());
    }

    // 2. Try Authorization header
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            let token = auth_str.trim_start_matches("Bearer ");
            return Ok(token.to_string());
        }
    }

    // 3. If neither is found, return an error
    Err(AuthError::MissingToken.into())
}

impl FromRequestParts<SharedState> for Session {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        // Get token
        let token = extract_token(parts, state)?;

        // Get session from token
        let session = queries::get_session_token_and_update_access_time(&state.db, &token)
            .await?
            .map(Self)
            .ok_or_else(|| AppError::from(AuthError::ExpiredToken))?;

        // Check if session is expired
        if session.0.expires_at < chrono::Utc::now() {
            return Err(AppError::from(AuthError::ExpiredToken));
        }

        Ok(session)
    }
}
