use axum::{routing::get, Router};

use crate::{handlers::role, server::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(role::list))
        .route("/roles/{id}", get(role::get))
}
