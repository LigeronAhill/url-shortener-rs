use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::post;
use axum::{extract::State, middleware, Json, Router};
use axum_auth::AuthBasic;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::server::AppState;
use crate::AppError;

pub fn url_routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(save_url))
        .route_layer(middleware::from_fn_with_state(state.clone(), check_auth))
        .with_state(state)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlResponse {
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}
pub async fn check_auth(
    State(state): State<AppState>,
    auth: Option<AuthBasic>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(a) = auth {
        let (user, password) = a.0;
        if user == state.config.http_server.user
            && password.is_some_and(|p| p == state.config.http_server.password)
        {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
#[instrument]
pub async fn save_url(
    State(state): State<AppState>,
    Json(body): Json<UrlRequest>,
) -> crate::Result<Json<UrlResponse>> {
    info!("Got request: {body:#?}");
    let url = url::Url::parse(&body.url).map_err(|_| AppError::UrlError)?;
    let alias = body
        .alias
        .unwrap_or_else(|| state.generator.generate_alias());
    if alias.len() < 4 {
        return Err(AppError::ToShortAlias);
    }
    let id = state.storage.save_url(url, alias.clone()).await?;
    info!("url saved with id: {id}");
    Ok(Json(UrlResponse {
        status: ResponseStatus::Ok,
        error: None,
        alias: Some(alias),
    }))
}

#[derive(Debug, Clone, Serialize)]
pub enum ResponseStatus {
    Ok,
    Error,
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use tokio::net::TcpListener;

    use crate::server::UrlRequest;

    #[tokio::test]
    async fn test_save_url() -> crate::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let address = listener.local_addr()?;
        tokio::spawn(async move { crate::test_run(listener).await.unwrap() });
        let uri = &format!("http://0.0.0.0:{}/url", address.port());
        let client = reqwest::Client::new();
        let body = UrlRequest {
            url: "https://google.com".to_string(),
            alias: None,
        };
        let response = client
            .post(uri)
            .basic_auth("test", Some("test"))
            .json(&body)
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let response = client.post(uri).json(&body).send().await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = UrlRequest {
            url: "google.com".to_string(),
            alias: None,
        };
        let response = client
            .post(uri)
            .json(&body)
            .basic_auth("test", Some("test"))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }
}
