use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts",
            post(handlers::employee_payroll_concept::create)
                .get(handlers::employee_payroll_concept::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}",
            get(handlers::employee_payroll_concept::get)
                .put(handlers::employee_payroll_concept::update)
                .delete(handlers::employee_payroll_concept::delete),
        )
}
