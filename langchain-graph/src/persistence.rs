//! Persistence backend for graph state.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;

#[async_trait]
pub trait BasePersistence: Send + Sync {
    async fn save_graph_state(&self, session_id: &str, state: &Value) -> Result<()>;
    async fn load_graph_state(&self, session_id: &str) -> Result<Option<Value>>;
    async fn list_sessions(&self) -> Result<Vec<String>>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
}

pub struct SQLitePersistence {
    pool: sqlx::SqlitePool,
}

impl SQLitePersistence {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS graph_state (
                session_id TEXT PRIMARY KEY,
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
impl BasePersistence for SQLitePersistence {
    async fn save_graph_state(&self, session_id: &str, state: &Value) -> Result<()> {
        let state_str =
            serde_json::to_string(state).map_err(|e| ChainError::ParserError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO graph_state (session_id, state) VALUES ($1, $2)
             ON CONFLICT(session_id) DO UPDATE SET state = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(session_id)
        .bind(&state_str)
        .execute(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }

    async fn load_graph_state(&self, session_id: &str) -> Result<Option<Value>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state FROM graph_state WHERE session_id = $1",
        )
        .bind(session_id)
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

    async fn list_sessions(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT session_id FROM graph_state")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM graph_state WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }
}

pub struct PostgresPersistence {
    pool: sqlx::PgPool,
}

impl PostgresPersistence {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(database_url)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS graph_state (
                session_id TEXT PRIMARY KEY,
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
impl BasePersistence for PostgresPersistence {
    async fn save_graph_state(&self, session_id: &str, state: &Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO graph_state (session_id, state) VALUES ($1, $2)
             ON CONFLICT(session_id) DO UPDATE SET state = $2, updated_at = NOW()",
        )
        .bind(session_id)
        .bind(state)
        .execute(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }

    async fn load_graph_state(&self, session_id: &str) -> Result<Option<Value>> {
        let row: Option<(Value,)> = sqlx::query_as(
            "SELECT state FROM graph_state WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT session_id FROM graph_state")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM graph_state WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::IOError(e.to_string()))?;

        Ok(())
    }
}
