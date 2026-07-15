use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use surrealdb::Error as SurrealError;
use thiserror::Error;
use uuid::Uuid;

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
    #[error("unauthorized: {message}")]
    Unauthorized { message: String },
    #[error("forbidden: {message}")]
    Forbidden { message: String },
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

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
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

impl From<uuid::Error> for AppError {
    fn from(value: uuid::Error) -> Self {
        Self::Internal {
            message: format!("invalid UUID: {value}"),
        }
    }
}

/// Helper to parse a SurrealDB `Thing.id` into a `Uuid`.
///
/// SurrealDB can store record IDs as either strings or native UUIDs,
/// so this handles both formats.
pub fn parse_thing_id(id: &surrealdb::types::RecordIdKey, context: &str) -> AppResult<Uuid> {
    use surrealdb::types::RecordIdKey;
    match id {
        RecordIdKey::String(value) => Uuid::parse_str(value)
            .map_err(|_| AppError::internal(format!("{context} is not a valid UUID"))),
        RecordIdKey::Uuid(value) => Ok((*value).into_inner()),
        _ => Err(AppError::internal(format!(
            "{context} is not a supported format"
        ))),
    }
}

/// Helper to parse a string field into a `Uuid`.
pub fn parse_uuid_field(value: &str, context: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| AppError::internal(format!("{context} is not a valid UUID")))
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Validation { message } => (StatusCode::UNPROCESSABLE_ENTITY, message.clone()),
            AppError::NotFound { message } => (StatusCode::NOT_FOUND, message.clone()),
            AppError::Conflict { message } => (StatusCode::CONFLICT, message.clone()),
            AppError::Database { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("database error: {message}"),
            ),
            AppError::Internal { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("internal server error: {message}"),
            ),
            AppError::Unauthorized { message } => (StatusCode::UNAUTHORIZED, message.clone()),
            AppError::Forbidden { message } => (StatusCode::FORBIDDEN, message.clone()),
        };

        let body = Json(ErrorBody { error: message });
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}
