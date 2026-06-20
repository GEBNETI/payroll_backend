use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers::auth, server::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
}
