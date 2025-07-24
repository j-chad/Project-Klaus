use crate::config;
use anyhow::Result;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::{debug, info};

pub async fn connect_db(config: &config::PostgreSQLSettings) -> Result<PgPool> {
    debug!(lazy = config.lazy, "Connecting to postgresql");
    let pool_options = PgPoolOptions::new().max_connections(config.max_connections);

    let pool = if config.lazy {
        pool_options.connect_lazy(&config.url)?
    } else {
        pool_options.connect(&config.url).await?
    };

    // Log success message
    info!("Successfully connected to postgresql");
    Ok(pool)
}
