use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(&'static str),
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "bad_request",
                    message,
                }),
            )
                .into_response(),
            Self::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "not_found",
                    message: format!("{resource} was not found"),
                }),
            )
                .into_response(),
            Self::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "internal_error",
                    message,
                }),
            )
                .into_response(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        Self::Internal(value.to_string())
    }
}
