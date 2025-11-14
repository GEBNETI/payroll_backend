use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions",
            post(handlers::division::create).get(handlers::division::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}",
            get(handlers::division::get)
                .put(handlers::division::update)
                .delete(handlers::division::delete),
        )
}
