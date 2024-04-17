use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use axum_auth::AuthBasic;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};

use crate::server::AppState;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    auth: Option<AuthBasic>,
    Json(body): Json<UrlRequest>,
) -> impl IntoResponse {
    info!("Got request: {body:#?}");
    if let Some(a) = auth {
        let (user, password) = a.0;
        if user == state.config.http_server.user
            && password.is_some_and(|p| p == state.config.http_server.password)
        {
            let alias = body
                .alias
                .unwrap_or_else(|| state.generator.generate_alias());
            if alias.len() < 4 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(UrlResponse {
                        status: Status::Error,
                        error: Some(String::from("alias length must be greater than 4")),
                        alias: None,
                    }),
                );
            }
            match state.storage.save_url(body.url, alias.clone()).await {
                Ok(id) => {
                    info!("url saved with id: {id}");
                    (
                        StatusCode::OK,
                        Json(UrlResponse {
                            status: Status::Ok,
                            error: None,
                            alias: Some(alias),
                        }),
                    )
                }
                Err(e) => {
                    error!("Error while saving url: {e:?}");
                    match e {
                        crate::AppError::Custom(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UrlResponse {
                                status: Status::Error,
                                error: Some(e.to_string()),
                                alias: None,
                            }),
                        ),
                        crate::AppError::UrlExists => (
                            StatusCode::BAD_REQUEST,
                            Json(UrlResponse {
                                status: Status::Error,
                                error: Some(String::from("alias already exists")),
                                alias: None,
                            }),
                        ),
                        _ => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(UrlResponse {
                                status: Status::Error,
                                error: Some(String::from("Database error")),
                                alias: None,
                            }),
                        ),
                    }
                }
            }
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(UrlResponse {
                    status: Status::Error,
                    error: Some(String::from("unauthorized")),
                    alias: None,
                }),
            );
        }
    } else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(UrlResponse {
                status: Status::Error,
                error: Some(String::from("unauthorized")),
                alias: None,
            }),
        );
    }
}
#[derive(Serialize, Debug, Clone)]
pub enum Status {
    Ok,
    Error,
}

#[cfg(test)]
mod tests {
    use crate::server::app;
    use crate::storage;
    use axum::body::Body;
    use axum::http;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_save_url() -> crate::Result<()> {
        let conf = crate::config::init("config/local.toml");
        let storage = storage::sqlite::init(&conf).await?;
        let app = app(&conf, storage);
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/url")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(
                        &json!({"url": "https://google.com"}),
                    )?))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }
}
