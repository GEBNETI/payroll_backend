use axum::{routing::post, Router};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate",
            post(handlers::payroll_calculator::calculate),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/clean",
            post(handlers::payroll_calculator::clean),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/recalculate",
            post(handlers::payroll_calculator::recalculate),
        )
}
