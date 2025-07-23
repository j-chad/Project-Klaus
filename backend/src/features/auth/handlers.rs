use super::schemas::{JoinRoomRequest, JoinRoomResponse};
use super::{middleware::Session, service, utils};
use crate::error::AppError;
use crate::features::auth::utils::new_session_cookie;
use crate::state::SharedState;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use axum_extra::extract::CookieJar;
use std::net::SocketAddr;
use validator::Validate;

pub async fn join_room(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookies: CookieJar,
    Json(body): Json<JoinRoomRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;

    let user_id = service::join_room(&state.db, &body).await?;

    let ip_address = addr.ip();
    let user_agent = headers.get("User-Agent").and_then(|h| h.to_str().ok());

    let session_token =
        service::create_session_token(&state.db, user_id, user_agent, Some(ip_address)).await?;
    let session_cookie = utils::new_session_cookie(&state.config.auth, &session_token);

    let connection_ticket =
        service::create_ephemeral_token(&state.db, user_id, user_agent, Some(ip_address)).await?;

    Ok((
        StatusCode::CREATED,
        cookies.add(session_cookie),
        Json(JoinRoomResponse { connection_ticket }),
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
