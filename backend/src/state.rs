use crate::config;
use sqlx::PgPool;
use std::sync::Arc;

pub struct AppState {
    pub db: PgPool,
    pub config: config::Settings,
}

impl AppState {
    pub fn new(db: PgPool, config: config::Settings) -> Self {
        AppState { db, config }
    }
}

pub type SharedState = Arc<AppState>;
