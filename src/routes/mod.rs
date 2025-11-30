use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{openapi::ApiDoc, server::AppState};

pub mod bank;
pub mod division;
pub mod employee;
pub mod employee_payroll_concept;
pub mod health;
pub mod job;
pub mod organization;
pub mod payroll;
pub mod payroll_concept;
pub mod payroll_concept_definition;

pub fn app_router(state: AppState) -> Router {
    let openapi = ApiDoc::openapi();

    Router::<AppState>::new()
        .merge(health::router())
        .merge(organization::router())
        .merge(payroll::router())
        .merge(payroll_concept::router())
        .merge(payroll_concept_definition::router())
        .merge(job::router())
        .merge(division::router())
        .merge(employee_payroll_concept::router())
        .merge(bank::router())
        .merge(employee::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}
