use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/banks",
            post(handlers::bank::create).get(handlers::bank::list),
        )
        .route(
            "/organizations/{organization_id}/banks/{bank_id}",
            get(handlers::bank::get)
                .put(handlers::bank::update)
                .delete(handlers::bank::delete),
        )
}
