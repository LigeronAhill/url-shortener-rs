use crate::{config::Configuration, AppError};
use anyhow::Result;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use tracing::{info, instrument};

#[derive(Debug)]
pub struct SqliteStorage {
    db: Pool<Sqlite>,
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
    pub async fn save_url<T: ToString + std::fmt::Debug>(
        &self,
        url_to_save: T,
        alias: T,
    ) -> crate::Result<i64> {
        match sqlx::query("INSERT INTO url(url, alias) VALUES (?, ?)")
            .bind(url_to_save.to_string())
            .bind(alias.to_string())
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
}

pub async fn init(config: &Configuration) -> Result<SqliteStorage> {
    let db_url = &config.storage_path;
    let db = SqlitePool::connect(db_url).await?;
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
