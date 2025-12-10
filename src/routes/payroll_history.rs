use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history",
            post(handlers::payroll_history::create).get(handlers::payroll_history::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}",
            get(handlers::payroll_history::get)
                .put(handlers::payroll_history::update)
                .delete(handlers::payroll_history::delete),
        )
}
