use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use surrealdb::Error as SurrealError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {message}")]
    Validation { message: String },
    #[error("resource not found: {message}")]
    NotFound { message: String },
    #[error("conflict: {message}")]
    Conflict { message: String },
    #[error("database error: {message}")]
    Database { message: String },
    #[error("internal server error: {message}")]
    Internal { message: String },
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

impl From<SurrealError> for AppError {
    fn from(value: SurrealError) -> Self {
        Self::Database {
            message: value.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Validation { message } => (StatusCode::UNPROCESSABLE_ENTITY, message.clone()),
            AppError::NotFound { message } => (StatusCode::NOT_FOUND, message.clone()),
            AppError::Conflict { message } => (StatusCode::CONFLICT, message.clone()),
            AppError::Database { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database error".to_string(),
            ),
            AppError::Internal { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };

        let body = Json(ErrorBody { error: message });
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}
