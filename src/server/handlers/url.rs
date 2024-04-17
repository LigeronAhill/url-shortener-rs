use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

use crate::server::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct UrlRequest {
    pub url: url::Url,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlResponse {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}
#[instrument]
pub async fn save_url(
    State(state): State<AppState>,
    Json(body): Json<UrlRequest>,
) -> impl IntoResponse {
    info!("Got request: {body:#?}");
    let alias = body
        .alias
        .unwrap_or_else(|| state.generator.generate_alias());
    match state.storage.save_url(body.url, alias.clone()).await {
        Ok(id) => {
            info!("url saved with id: {id}");
            Json(UrlResponse {
                status: Status::Ok,
                error: None,
                alias: Some(alias),
            })
        }
        Err(e) => {
            error!("Error while saving url: {e:?}");
            match e {
                crate::AppError::Custom(e) => Json(UrlResponse {
                    status: Status::Error,
                    error: Some(e.to_string()),
                    alias: None,
                }),
                crate::AppError::UrlExists => Json(UrlResponse {
                    status: Status::Error,
                    error: Some(String::from("alias already exists")),
                    alias: None,
                }),
                _ => Json(UrlResponse {
                    status: Status::Error,
                    error: Some(String::from("Database error")),
                    alias: None,
                }),
            }
        }
    }
}
#[derive(Serialize, Debug, Clone)]
pub enum Status {
    Ok,
    Error,
}
