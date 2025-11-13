#[path = "support/mod.rs"]
mod support;

use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

fn read_json(body: Bytes) -> Value {
    serde_json::from_slice(&body).expect("valid json")
}

#[tokio::test]
async fn creating_an_organization_returns_created_payload() {
    let app = support::test_router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/organizations")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Acme"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(payload["name"], "Acme");
    let id = payload["id"].as_str().expect("id");
    Uuid::parse_str(id).expect("uuid in response");
}

#[tokio::test]
async fn listing_organizations_returns_sorted_results() {
    let app = support::test_router();

    for name in ["Zed", "Acme"] {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/organizations")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"name": name}).to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/organizations")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let items = payload.as_array().expect("array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], "Acme");
    assert_eq!(items[1]["name"], "Zed");
}

#[tokio::test]
async fn can_update_and_delete_an_organization() {
    let app = support::test_router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/organizations")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Acme"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/organizations/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Acme Two"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["name"], "Acme Two");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/organizations/{id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/organizations/{id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejecting_empty_names() {
    let app = support::test_router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/organizations")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "   "}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
