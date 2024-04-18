use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;
use tracing::instrument;

use crate::server::AppState;
pub fn redirect_routes(state: AppState) -> Router {
    Router::new()
        .route("/:alias", get(redirect_url))
        .with_state(state)
}
#[instrument]
pub async fn redirect_url(
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> crate::Result<impl IntoResponse> {
    let url = state.storage.get_url(alias).await?;
    Ok(Redirect::temporary(&url))
}
