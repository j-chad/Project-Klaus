pub struct TokenStore {
    pool: sqlx::SqlitePool,
}

impl TokenStore {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<(), TokenError> {
        // Create tokens table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS ephemeral_tokens (
                token TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                used_at INTEGER NULL
            )
            "#
        )
            .execute(&self.pool)
            .await?;

        // Create index for efficient cleanup and lookups
        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_tokens_expires_at ON ephemeral_tokens(expires_at)"
        )
            .execute(&self.pool)
            .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_tokens_user_id ON ephemeral_tokens(user_id)"
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}