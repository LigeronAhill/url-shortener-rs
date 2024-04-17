use axum::async_trait;

pub mod sqlite;

#[async_trait]
pub trait Repository: Send + Sync + std::fmt::Debug {
    async fn save_url(&self, url_to_save: url::Url, alias: String,) -> crate::Result<i64>;
}