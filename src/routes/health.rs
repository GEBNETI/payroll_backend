use axum::{Router, routing::get};

use crate::handlers;

pub fn router() -> Router {
    Router::new().route("/health", get(handlers::health::check))
}
