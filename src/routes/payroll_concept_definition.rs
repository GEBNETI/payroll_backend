use axum::{routing::post, Router};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route(
        "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition",
        post(handlers::payroll_concept_definition::create)
            .get(handlers::payroll_concept_definition::get)
            .put(handlers::payroll_concept_definition::update)
            .delete(handlers::payroll_concept_definition::delete),
    )
}
