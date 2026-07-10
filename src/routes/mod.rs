use axum::{
    http::{header, Method},
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
pub mod payroll_history_report;
pub mod user;

fn build_cors() -> CorsLayer {
    CorsLayer::new()
        // A credentialed browser request cannot use `Access-Control-Allow-Origin: *`.
        // Mirroring the request origin provides the equivalent any-origin behavior while
        // allowing the refresh-token cookie to be sent from every frontend deployment.
        .allow_origin(AllowOrigin::mirror_request())
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
        .merge(payroll_history_report::router())
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    use super::build_cors;

    #[tokio::test]
    async fn cors_mirrors_any_origin_for_credentialed_preflight_requests() {
        let app = Router::new()
            .route("/auth/refresh", post(|| async { StatusCode::OK }))
            .layer(build_cors());
        let origin = "https://payroll.velociraptor-reedfish.ts.net";
        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/auth/refresh")
                    .header(header::ORIGIN, origin)
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some(origin)
        );
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .and_then(|value| value.to_str().ok()),
            Some("true")
        );
    }
}
