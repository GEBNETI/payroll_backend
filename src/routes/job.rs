use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/jobs",
            post(handlers::job::create).get(handlers::job::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}",
            get(handlers::job::get)
                .put(handlers::job::update)
                .delete(handlers::job::delete),
        )
}
