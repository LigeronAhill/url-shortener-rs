use std::borrow::Cow;
use std::sync::Arc;

use axum::{error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, Router};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

pub use handlers::url::{ResponseStatus, UrlRequest, UrlResponse};

use crate::server::handlers::redirect::redirect_routes;
use crate::server::handlers::url::url_routes;
use crate::storage::Repository;
use crate::{config::Configuration, gen_alias::Generator};

mod handlers;

pub fn app(config: &Configuration, storage: impl Repository + 'static) -> Router {
    let state = AppState::new(storage, config);
    Router::new()
        .nest("/", redirect_routes(state.clone()))
        .nest("/url", url_routes(state.clone()))
        // .route("/url", post(handlers::url::save_url))
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
    pub storage: Arc<dyn Repository>,
    pub config: Configuration,
}
impl AppState {
    pub fn new(storage: impl Repository + 'static, config: &Configuration) -> Self {
        let generator = Generator::new(config);
        Self {
            storage: Arc::new(storage),
            generator,
            config: config.clone(),
        }
    }
}
