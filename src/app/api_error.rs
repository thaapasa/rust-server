use std::error::Error;
use std::fmt::Display;

use axum::http::{header, StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use tracing::error;

use crate::error::InternalError;

#[derive(Debug)]
pub struct ApiError {
    http_status: StatusCode,
    body: Value,
}

impl ApiError {
    pub fn not_found() -> Self {
        Self {
            http_status: StatusCode::NOT_FOUND,
            body: json!({ "error": "not_found" }),
        }
    }
    pub fn invalid_path_param() -> Self {
        Self {
            http_status: StatusCode::BAD_REQUEST,
            body: json!({ "error": "invalid_path_param" }),
        }
    }
    pub fn internal(e: InternalError) -> Self {
        error!("{e}");
        Self {
            http_status: StatusCode::INTERNAL_SERVER_ERROR,
            body: json!({ "error": "internal_server_error" }),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ApiError {}", self.http_status))
    }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.http_status,
            [(header::CONTENT_TYPE, "application/json")],
            Json(self.body),
        )
            .into_response()
    }
}

impl From<InternalError> for ApiError {
    fn from(e: InternalError) -> Self {
        ApiError::internal(e)
    }
}
