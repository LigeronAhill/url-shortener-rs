use crate::server::{ResponseStatus, UrlResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub enum AppError {
    Custom(anyhow::Error),
    UrlNotFound,
    UrlExists,
    DatabaseError,
    ToShortAlias,
    UrlError,
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Custom(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UrlResponse {
                    status: ResponseStatus::Error,
                    error: Some(format!("{e:?}")),
                    alias: None,
                }),
            )
                .into_response(),
            AppError::UrlNotFound | AppError::UrlExists => (
                StatusCode::BAD_REQUEST,
                Json(UrlResponse {
                    status: ResponseStatus::Error,
                    error: Some("alias already exists".to_string()),
                    alias: None,
                }),
            )
                .into_response(),
            AppError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UrlResponse {
                    status: ResponseStatus::Error,
                    error: Some("database error".to_string()),
                    alias: None,
                }),
            )
                .into_response(),
            AppError::ToShortAlias => (
                StatusCode::BAD_REQUEST,
                Json(UrlResponse {
                    status: ResponseStatus::Error,
                    error: Some("alias length must be greater than 4".to_string()),
                    alias: None,
                }),
            )
                .into_response(),
            AppError::UrlError => (
                StatusCode::BAD_REQUEST,
                Json(UrlResponse {
                    status: ResponseStatus::Error,
                    error: Some("incorrect url".to_string()),
                    alias: None,
                }),
            )
                .into_response(),
        }
    }
}
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Custom(err.into())
    }
}
pub type Result<T> = core::result::Result<T, AppError>;
