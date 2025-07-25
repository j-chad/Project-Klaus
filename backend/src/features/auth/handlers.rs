use super::schemas;
use super::{middleware::Session, service};
use crate::error::AppError;
use crate::features::auth::utils::new_session_cookie;
use crate::state::SharedState;
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use std::net::SocketAddr;

pub async fn create_challenge(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<schemas::CreateChallengeTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());
    let ip_address = Some(addr.ip());

    let challenge_token =
        service::create_challenge_token(&state.db, &request.fingerprint, user_agent, ip_address)
            .await?;

    Ok((
        StatusCode::CREATED,
        Json(schemas::ChallengeResponse {
            challenge: challenge_token,
        }),
    ))
}

pub async fn verify_challenge(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookies: CookieJar,
    Json(request): Json<schemas::ChallengeVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());
    let ip_address = Some(addr.ip());

    let session_token = service::exchange_challenge_for_session(
        &state.db,
        &request.token,
        &request.fingerprint,
        user_agent,
        ip_address,
    )
    .await?;

    let session_cookie = new_session_cookie(&state.config.auth, &session_token);
    Ok((StatusCode::CREATED, cookies.add(session_cookie)))
}

pub async fn create_ephemeral_token(
    State(state): State<SharedState>,
    Session(session): Session,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());
    let ip_address = Some(addr.ip());

    let ephemeral_token =
        service::create_ephemeral_token(&state.db, session.member_id, user_agent, ip_address)
            .await?;

    Ok((
        StatusCode::CREATED,
        Json(schemas::EphemeralTokenResponse { ephemeral_token }),
    ))
}

pub async fn logout(
    State(state): State<SharedState>,
    Session(session): Session,
    cookies: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    service::logout(&state.db, session.member_id).await?;

    let removal_cookie = new_session_cookie(&state.config.auth, "");
    Ok((StatusCode::NO_CONTENT, cookies.remove(removal_cookie)))
}
