//! PostgreSQL-based checkpointer for persistent graph state.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;

use crate::checkpoint::Checkpointer;

pub struct PostgresCheckpointer {
    pool: sqlx::PgPool,
}

impl PostgresCheckpointer {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(database_url)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                state JSONB NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn from_pool(pool: sqlx::PgPool) -> Result<Self> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                state JSONB NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Checkpointer for PostgresCheckpointer {
    async fn save(&self, checkpoint_id: &str, state: &Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO checkpoints (checkpoint_id, state) VALUES ($1, $2)
             ON CONFLICT(checkpoint_id) DO UPDATE SET state = $2, updated_at = NOW()",
        )
        .bind(checkpoint_id)
        .bind(state)
        .execute(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, checkpoint_id: &str) -> Result<Option<Value>> {
        let row: Option<(Value,)> = sqlx::query_as(
            "SELECT state FROM checkpoints WHERE checkpoint_id = $1",
        )
        .bind(checkpoint_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    async fn list(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT checkpoint_id FROM checkpoints")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn delete(&self, checkpoint_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM checkpoints WHERE checkpoint_id = $1")
            .bind(checkpoint_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }
}
