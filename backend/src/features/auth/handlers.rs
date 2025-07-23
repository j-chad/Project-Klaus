use super::schemas::{JoinRoomRequest, JoinRoomResponse};
use super::{service, utils};
use crate::error::AppError;
use crate::state::SharedState;
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use std::net::SocketAddr;
use validator::Validate;

pub async fn join_room(
    State(state): State<SharedState>,
    Json(body): Json<JoinRoomRequest>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookies: CookieJar,
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
