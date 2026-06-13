//! SQLite-based checkpointer for persistent graph state.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;

use crate::checkpoint::Checkpointer;

pub struct SQLiteCheckpointer {
    pool: sqlx::SqlitePool,
}

impl SQLiteCheckpointer {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn from_pool(pool: sqlx::SqlitePool) -> Result<Self> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Checkpointer for SQLiteCheckpointer {
    async fn save(&self, checkpoint_id: &str, state: &Value) -> Result<()> {
        let state_str =
            serde_json::to_string(state).map_err(|e| ChainError::ParserError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO checkpoints (checkpoint_id, state) VALUES ($1, $2)
             ON CONFLICT(checkpoint_id) DO UPDATE SET state = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(checkpoint_id)
        .bind(&state_str)
        .execute(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, checkpoint_id: &str) -> Result<Option<Value>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state FROM checkpoints WHERE checkpoint_id = $1",
        )
        .bind(checkpoint_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        match row {
            Some((state_str,)) => {
                let value: Value = serde_json::from_str(&state_str)
                    .map_err(|e| ChainError::ParserError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
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
