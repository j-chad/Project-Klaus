use super::{schemas, service};
use crate::error::AppError;
use crate::features::auth;
use crate::state::SharedState;
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use std::net::SocketAddr;
use validator::Validate;

pub async fn create_room(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookies: CookieJar,
    Json(body): Json<schemas::CreateRoomRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;

    let (user_id, room_code) = service::create_room(
        &state.db,
        &body.room_name,
        &body.username,
        &body.public_key,
        &body.seed_hash,
        body.max_players,
    )
    .await?;

    let ip_address = Some(addr.ip());
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());

    let session_token =
        auth::service::create_session_token(&state.db, user_id, user_agent, ip_address).await?;
    let session_cookie =
        auth::utils::cookie::new_session_cookie(&state.config.auth, &session_token);

    let ephemeral_token =
        auth::service::create_ephemeral_token(&state.db, user_id, user_agent, ip_address).await?;

    Ok((
        StatusCode::CREATED,
        cookies.add(session_cookie),
        Json(schemas::CreateRoomResponse {
            room_id: room_code,
            ephemeral_token,
        }),
    ))
}

pub async fn join_room(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookies: CookieJar,
    Json(body): Json<schemas::JoinRoomRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;

    let user_id = service::join_room(
        &state.db,
        &body.room_id,
        &body.name,
        &body.public_key,
        &body.seed_hash,
    )
    .await?;

    let ip_address = Some(addr.ip());
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());

    let session_token =
        auth::service::create_session_token(&state.db, user_id, user_agent, ip_address).await?;
    let session_cookie =
        auth::utils::cookie::new_session_cookie(&state.config.auth, &session_token);

    let ephemeral_token =
        auth::service::create_ephemeral_token(&state.db, user_id, user_agent, ip_address).await?;

    Ok((
        StatusCode::CREATED,
        cookies.add(session_cookie),
        Json(auth::schemas::EphemeralTokenResponse { ephemeral_token }),
    ))
}

pub async fn start_game(
    State(state): State<SharedState>,
    auth::Session(session): auth::Session,
    Json(body): Json<schemas::SantaIDMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::requires_owner_permission(&state.db, &session.member_id).await?;

    service::start_game(&state.db, &session.member_id).await?;
    service::handle_santa_id_message(&state.db, &session.member_id, &body.message_content).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_santa_id_message(
    State(state): State<SharedState>,
    auth::Session(session): auth::Session,
    Json(body): Json<schemas::SantaIDMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::handle_santa_id_message(&state.db, &session.member_id, &body.message_content).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_seed_reveal(
    State(state): State<SharedState>,
    auth::Session(session): auth::Session,
    Json(body): Json<schemas::SeedRevealRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::reveal_seed(&state.db, &session.member_id, &body.seed).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_verification(
    State(state): State<SharedState>,
    auth::Session(session): auth::Session,
    Json(body): Json<schemas::VerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::handle_verification(&state.db, &session.member_id, &body).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_rejection_ack(
    State(state): State<SharedState>,
    auth::Session(session): auth::Session,
    Json(body): Json<schemas::ResultAckRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::acknowledge_rejection(&state.db, &session.member_id, &body.seed_hash).await?;

    Ok(StatusCode::NO_CONTENT)
}
