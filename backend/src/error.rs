//! Error types and handling for the Eigenix backend API
//!
//! This module provides a unified error type that implements `IntoResponse`
//! for proper HTTP error responses in Axum handlers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// API error response sent to clients
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

/// Main error type for the Eigenix backend
#[derive(Debug)]
pub enum ApiError {
    /// Database operation failed
    Database(anyhow::Error),
    /// Wallet operation failed
    Wallet(anyhow::Error),
    /// Metrics collection failed
    Metrics(anyhow::Error),
    /// Resource not found
    NotFound(String),
    /// Invalid input/request
    BadRequest(String),
    /// Internal server error
    Internal(anyhow::Error),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(e) => write!(f, "Database error: {}", e),
            ApiError::Wallet(e) => write!(f, "Wallet error: {}", e),
            ApiError::Metrics(e) => write!(f, "Metrics error: {}", e),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApiError::Database(e) | ApiError::Wallet(e) | ApiError::Metrics(e) | ApiError::Internal(e) => {
                e.source()
            }
            ApiError::NotFound(_) | ApiError::BadRequest(_) => None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            ApiError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database operation failed".to_string(),
                Some(e.to_string()),
            ),
            ApiError::Wallet(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Wallet operation failed".to_string(),
                Some(e.to_string()),
            ),
            ApiError::Metrics(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Metrics collection failed".to_string(),
                Some(e.to_string()),
            ),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "Resource not found".to_string(), Some(msg)),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad request".to_string(), Some(msg)),
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                Some(e.to_string()),
            ),
        };

        // Log the error
        tracing::error!("{}: {:?}", error_message, details);

        let body = Json(ErrorResponse {
            error: error_message,
            details,
        });

        (status, body).into_response()
    }
}

/// Convenience result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

/// Convert from anyhow::Error to ApiError::Internal
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}

