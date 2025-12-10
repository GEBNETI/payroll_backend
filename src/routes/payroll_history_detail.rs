use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details",
            post(handlers::payroll_history_detail::create)
                .get(handlers::payroll_history_detail::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details/{detail_id}",
            get(handlers::payroll_history_detail::get)
                .delete(handlers::payroll_history_detail::delete),
        )
}
