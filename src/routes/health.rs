use axum::{Router, routing::get};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/health", get(handlers::health::check))
}
