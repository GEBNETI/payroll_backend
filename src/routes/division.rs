use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/divisions",
            post(handlers::division::create).get(handlers::division::list),
        )
        .route(
            "/divisions/{id}",
            get(handlers::division::get)
                .put(handlers::division::update)
                .delete(handlers::division::delete),
        )
}
