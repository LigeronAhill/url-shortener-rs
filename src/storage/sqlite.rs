use crate::{config::Configuration, AppError};
use anyhow::Result;
use axum::async_trait;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Row, Sqlite, SqlitePool};
use tracing::{info, instrument};
use crate::storage::Repository;

#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pub db: Pool<Sqlite>,
}
impl SqliteStorage {
    pub async fn check(&self) -> Result<()> {
        let sql = "
        SELECT name
        FROM sqlite_schema
        WHERE type = 'table'
        AND name NOT LIKE 'sqlite_%';
        ";
        let qr = sqlx::query(sql).fetch_all(&self.db).await?;
        qr.iter().enumerate().for_each(|(index, row)| {
            info!("[{index}]: {:?}", row.get::<String, &str>("name"));
        });
        Ok(())
    }
    #[instrument]
    pub async fn get_url<T: ToString + std::fmt::Debug>(&self, alias: T) -> crate::Result<String> {
        let qr = sqlx::query("SELECT url FROM url WHERE alias = ?")
            .bind(alias.to_string())
            .fetch_optional(&self.db)
            .await?
            .ok_or(AppError::UrlNotFound)?;
        let x: String = qr.get("url");
        Ok(x)
    }
    #[instrument]
    pub async fn delete_url<T: ToString + std::fmt::Debug>(&self, alias: T) -> crate::Result<()> {
        let qr = sqlx::query("DELETE FROM url WHERE alias = ?")
            .bind(alias.to_string())
            .execute(&self.db)
            .await?;
        if qr.rows_affected() == 0 {
            Err(AppError::UrlNotFound)
        } else {
            Ok(())
        }
    }
}
#[async_trait]
impl Repository for SqliteStorage {

    #[instrument]
    async fn save_url(
        &self,
        url_to_save: url::Url,
        alias: String,
    ) -> crate::Result<i64> {
        match sqlx::query("INSERT INTO url(url, alias) VALUES (?, ?)")
            .bind(url_to_save.to_string())
            .bind(alias)
            .execute(&self.db)
            .await
        {
            Ok(x) => Ok(x.last_insert_rowid()),
            Err(e) => {
                if e.as_database_error()
                    .is_some_and(|e| e.is_unique_violation())
                {
                    Err(AppError::UrlExists)
                } else {
                    Err(AppError::DatabaseError)
                }
            }
        }
    }
}

pub async fn init(config: &Configuration) -> Result<SqliteStorage> {
    let options = SqliteConnectOptions::new()
        .filename(&config.storage_path)
        .create_if_missing(true);
    let db = SqlitePool::connect_with(options).await?;
    let sql = r"
        CREATE TABLE IF NOT EXISTS url(
    		id INTEGER PRIMARY KEY,
    		alias TEXT NOT NULL UNIQUE,
    		url TEXT NOT NULL
    	);
    	CREATE INDEX IF NOT EXISTS idx_alias ON url(alias);
    ";
    let qr = sqlx::query(sql).execute(&db).await?;
    info!("Create url table result: {qr:#?}");
    Ok(SqliteStorage { db })
}
