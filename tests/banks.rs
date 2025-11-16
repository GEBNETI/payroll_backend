#[path = "support/mod.rs"]
mod support;

use axum::{
    Router,
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

async fn create_organization(app: &Router) -> Uuid {
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
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    Uuid::parse_str(body["id"].as_str().unwrap()).expect("uuid")
}

fn read_json(body: Bytes) -> Value {
    serde_json::from_slice(&body).expect("json")
}

#[tokio::test]
async fn can_create_and_list_banks() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;

    for name in ["Zed Bank", "Acme Bank"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/organizations/{organization_id}/banks"))
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "name": name }).to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/organizations/{organization_id}/banks"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let items = payload.as_array().expect("array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], "Acme Bank");
    assert_eq!(items[1]["name"], "Zed Bank");
}

#[tokio::test]
async fn can_update_and_delete_bank() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/banks"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "name": "Acme Bank" }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let bank_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/organizations/{organization_id}/banks/{bank_id}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "name": "Acme Bank Intl" }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["name"], "Acme Bank Intl");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/organizations/{organization_id}/banks/{bank_id}"))
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
                .uri(format!("/organizations/{organization_id}/banks/{bank_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_invalid_payloads_and_ownership() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let other_org = Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/banks"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "name": "   " }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{other_org}/banks"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "name": "Ghost Bank" }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
