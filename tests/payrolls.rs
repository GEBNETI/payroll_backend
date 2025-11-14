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
async fn can_create_and_list_payrolls() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "May 2024",
                        "description": "Monthly payroll"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["name"], "May 2024");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/organizations/{organization_id}/payrolls"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let list = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["organization_id"], organization_id.to_string());
}

#[tokio::test]
async fn rejects_invalid_organization_reference() {
    let app = support::test_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{}/payrolls", Uuid::new_v4()))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Invalid",
                        "description": "bad"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn can_update_and_delete_payroll() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "June",
                        "description": "June payroll"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let payroll_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "June Updated",
                        "description": "Updated",
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["name"], "June Updated");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}"
                ))
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
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
