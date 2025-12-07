use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/concepts",
            post(handlers::payroll_concept::create).get(handlers::payroll_concept::list),
        )
        .route(
            "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}",
            get(handlers::payroll_concept::get)
                .put(handlers::payroll_concept::update)
                .delete(handlers::payroll_concept::delete),
        )
}
