#[path = "support/mod.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_package_metadata() {
    let app = support::test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request body"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");

    assert_eq!(body["application"], env!("CARGO_PKG_NAME"));
    assert_eq!(body["authors"], env!("CARGO_PKG_AUTHORS"));
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}
