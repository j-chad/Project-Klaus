use crate::state::SharedState;
use axum::extract::ws::{WebSocket};
use axum::extract::{Query, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::{any};
use serde::Deserialize;
use tracing::{trace};

pub fn build_router() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/ws", any(handler))
}

#[derive(Deserialize)]
struct WebsocketOptions {
    room: String,
    token: String,
}

async fn handler(ws: WebSocketUpgrade, options: Query<WebsocketOptions>) -> Response {
    trace!("handling WebSocket upgrade request");
    

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    trace!("websocket client connected");

    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}