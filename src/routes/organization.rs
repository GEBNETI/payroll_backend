use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations",
            post(handlers::organization::create).get(handlers::organization::list),
        )
        .route(
            "/organizations/{id}",
            get(handlers::organization::get)
                .put(handlers::organization::update)
                .delete(handlers::organization::delete),
        )
}
