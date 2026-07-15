use axum::{routing::get, Router};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/earnings-deductions",
            get(handlers::payroll_history_report::earnings_deductions),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/payroll",
            get(handlers::payroll_history_report::payroll),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/patria",
            get(handlers::payroll_history_report::patria),
        )
}
