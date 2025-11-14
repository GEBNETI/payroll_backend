use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls",
            post(handlers::payroll::create).get(handlers::payroll::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}",
            get(handlers::payroll::get)
                .put(handlers::payroll::update)
                .delete(handlers::payroll::delete),
        )
}
