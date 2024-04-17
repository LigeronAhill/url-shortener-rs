mod handlers;
use std::borrow::Cow;

use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, routing::post,
    Router,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

use crate::{config::Configuration, gen_alias::Generator, storage::sqlite::SqliteStorage};

pub fn app(config: &Configuration, storage: SqliteStorage) -> Router {
    let state = AppState::new(storage, config);
    Router::new()
        .route("/url", post(handlers::url::save_url))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(std::time::Duration::from_secs(
                    config.http_server.timeout as u64,
                ))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state)
}
async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {error}")),
    )
}
#[derive(Debug, Clone)]
pub struct AppState {
    pub generator: Generator,
    pub storage: SqliteStorage,
}
impl AppState {
    pub fn new(storage: SqliteStorage, config: &Configuration) -> Self {
        let generator = Generator::new(config);
        Self { storage, generator }
    }
}
