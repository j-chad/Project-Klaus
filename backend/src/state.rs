use crate::config;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub db: SqlitePool,
    pub config: config::Settings,
}

impl AppState {
    pub fn new(db: SqlitePool, config: config::Settings) -> Self {
        AppState { db, config }
    }
}

pub type SharedState = Arc<AppState>;