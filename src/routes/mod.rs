use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{openapi::ApiDoc, server::AppState};

pub mod auth;
pub mod bank;
pub mod role;
pub mod division;
pub mod employee;
pub mod employee_payroll_concept;
pub mod health;
pub mod job;
pub mod organization;
pub mod payroll;
pub mod payroll_calculator;
pub mod payroll_concept;
pub mod payroll_concept_definition;
pub mod payroll_history;
pub mod payroll_history_detail;
pub mod user;

fn build_cors() -> CorsLayer {
    let allow_origin = match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins) => {
            let parsed: Vec<HeaderValue> = origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();
            AllowOrigin::list(parsed)
        }
        // When no env var is set, mirror the request origin so credentials work in dev
        Err(_) => AllowOrigin::mirror_request(),
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true)
}

pub fn app_router(state: AppState) -> Router {
    let openapi = ApiDoc::openapi();

    Router::<AppState>::new()
        .merge(auth::router())
        .merge(user::router())
        .merge(role::router())
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
        .merge(payroll_history::router())
        .merge(payroll_history_detail::router())
        .merge(payroll_calculator::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(build_cors())

        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}
