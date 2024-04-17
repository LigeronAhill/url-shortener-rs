use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use tracing::instrument;

use crate::server::AppState;

#[instrument]
pub async fn redirect_url(
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> crate::Result<impl IntoResponse> {
    let url = state.storage.get_url(alias).await?;
    Ok(Redirect::temporary(&url))
}
