use std::path::Path;

use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tracing::info;

#[derive(Clone)]
pub struct Runtime {
    pool: SqlitePool,
}

impl Runtime {
    pub async fn open(state_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(state_dir)?;

        let path = state_dir.join("runtime.sqlite3");
        let url = format!("sqlite://{}", path.display());

        if !Sqlite::database_exists(&url).await.unwrap_or(false) {
            Sqlite::create_database(&url).await?;
        }

        let pool = SqlitePool::connect(&url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS runtime (
              key        TEXT PRIMARY KEY,
              value      TEXT NOT NULL,
              updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        info!(path = %path.display(), "opened runtime state");

        Ok(Self { pool })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_scalar::<_, String>("SELECT value FROM runtime WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO runtime (key, value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_value_survives_reopening() {
        let dir = std::env::temp_dir().join("missiond-runtime-test");
        let _ = std::fs::remove_dir_all(&dir);

        Runtime::open(&dir)
            .await
            .unwrap()
            .set("screen_on", "true")
            .await
            .unwrap();

        assert_eq!(
            Runtime::open(&dir).await.unwrap().get("screen_on").await.unwrap(),
            Some("true".to_string())
        );
    }

    #[tokio::test]
    async fn an_absent_key_is_none_rather_than_an_error() {
        let dir = std::env::temp_dir().join("missiond-runtime-absent");
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(Runtime::open(&dir).await.unwrap().get("nothing").await.unwrap(), None);
    }
}
