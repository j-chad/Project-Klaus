use crate::error::AppError;
use crate::features::auth;
use crate::features::room::WebsocketOptions;
use crate::state::SharedState;
use axum::extract::ws::WebSocket;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use tracing::trace;

pub async fn upgrade_handler(
    ws: WebSocketUpgrade,
    options: Query<WebsocketOptions>,
    State(state): State<SharedState>,
) -> Result<Response, AppError> {
    trace!("websocket connection request for room: {}", options.room);

    auth::service::validate_websocket_token(&state.db, &options.token, &options.room).await?;

    Ok(ws.on_upgrade(handle_socket))
}

async fn handle_socket(mut socket: WebSocket) {
    trace!("websocket client connected");

    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
