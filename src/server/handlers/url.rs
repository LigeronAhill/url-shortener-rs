use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::server::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct UrlRequest {
    pub url: url::Url,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}
#[instrument]
pub async fn save_url(
    State(state): State<AppState>,
    Json(body): Json<UrlRequest>,
) -> crate::Result<impl IntoResponse> {
    info!("Got request: {body:#?}");
    let alias = body
        .alias
        .unwrap_or_else(|| state.generator.generate_alias());
    let id = state.storage.save_url(body.url, &alias).await?;
    info!("url saved with id: {id}");
    Ok(Json(UrlResponse {
        status: StatusCode::OK.to_string(),
        error: None,
        alias: Some(alias),
    }))
}
