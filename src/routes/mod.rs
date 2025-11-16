use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{openapi::ApiDoc, server::AppState};

pub mod division;
pub mod health;
pub mod job;
pub mod organization;
pub mod payroll;

pub fn app_router(state: AppState) -> Router {
    let openapi = ApiDoc::openapi();

    Router::<AppState>::new()
        .merge(health::router())
        .merge(organization::router())
        .merge(payroll::router())
        .merge(job::router())
        .merge(division::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .with_state(state)
}
