use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees",
            post(handlers::employee::create).get(handlers::employee::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}",
            get(handlers::employee::get)
                .put(handlers::employee::update)
                .delete(handlers::employee::delete),
        )
}
