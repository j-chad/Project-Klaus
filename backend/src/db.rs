use crate::config;
use anyhow::Result;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::{debug, info};

pub async fn connect_db(config: &config::SQLiteSettings) -> Result<SqlitePool> {
    debug!(lazy = config.lazy, "Connecting to sqlite");
    let pool_options = SqlitePoolOptions::new().max_connections(config.max_connections);

    let pool = if config.lazy {
        pool_options.connect_lazy(&config.url)?
    } else {
        pool_options.connect(&config.url).await?
    };

    // Log success message
    info!("Successfully connected to sqlite");
    Ok(pool)
}