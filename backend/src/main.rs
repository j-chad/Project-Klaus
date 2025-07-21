use crate::app::create_app;
use crate::db::connect_db;
use crate::state::{AppState, SharedState};
use std::net::SocketAddr;
use std::sync::Arc;

mod app;
mod config;
mod db;
mod features;
mod logging;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Settings::new()?;
    logging::init_tracing(&config.logging)?;

    tracing::info!("Starting application in {} environment", config.env);
    let db = connect_db(&config.postgresql).await?;
    let app_state: SharedState = Arc::new(AppState::new(db, config.clone()));

    let app = create_app(app_state);
    let addr = format!("{}:{}", config.app.bind_address, config.app.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
