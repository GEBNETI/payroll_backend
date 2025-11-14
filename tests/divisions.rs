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

fn read_json(body: Bytes) -> Value {
    serde_json::from_slice(&body).expect("json")
}

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
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());
    Uuid::parse_str(payload["id"].as_str().unwrap()).expect("uuid")
}

async fn create_payroll(app: &Router, organization_id: Uuid) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/payrolls")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "May",
                        "description": "Monthly",
                        "organization_id": organization_id,
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());
    Uuid::parse_str(payload["id"].as_str().unwrap()).expect("uuid")
}

async fn create_division(
    app: &Router,
    payroll_id: Uuid,
    name: &str,
    parent: Option<Uuid>,
) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/divisions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": name,
                        "description": format!("Desc {name}"),
                        "budget_code": format!("BC-{name}"),
                        "payroll_id": payroll_id,
                        "parent_division_id": parent,
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response.into_body().collect().await.unwrap().to_bytes())
}

#[tokio::test]
async fn can_create_and_list_divisions() {
    let app = support::test_router();
    let org = create_organization(&app).await;
    let payroll = create_payroll(&app, org).await;

    let parent = create_division(&app, payroll, "Parent", None).await;
    let parent_id = parent["id"].as_str().unwrap();

    let child = create_division(
        &app,
        payroll,
        "Child",
        Some(Uuid::parse_str(parent_id).unwrap()),
    )
    .await;
    assert_eq!(child["parent_division_id"], parent_id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/divisions")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let list = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(list.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn rejects_invalid_parent_or_payroll() {
    let app = support::test_router();
    let org = create_organization(&app).await;
    let payroll_a = create_payroll(&app, org).await;
    let payroll_b = create_payroll(&app, org).await;

    // Missing payroll fails
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/divisions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Invalid",
                        "description": "desc",
                        "budget_code": "BC",
                        "payroll_id": Uuid::new_v4(),
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let parent = create_division(&app, payroll_a, "Parent", None).await;
    let parent_id = Uuid::parse_str(parent["id"].as_str().unwrap()).unwrap();

    // Parent must belong to same payroll
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/divisions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Bad",
                        "description": "desc",
                        "budget_code": "BC",
                        "payroll_id": payroll_b,
                        "parent_division_id": parent_id,
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn can_update_and_clear_parent() {
    let app = support::test_router();
    let org = create_organization(&app).await;
    let payroll = create_payroll(&app, org).await;

    let parent = create_division(&app, payroll, "Parent", None).await;
    let parent_id = parent["id"].as_str().unwrap();
    let child = create_division(
        &app,
        payroll,
        "Child",
        Some(Uuid::parse_str(parent_id).unwrap()),
    )
    .await;
    let child_id = child["id"].as_str().unwrap();

    // Clear parent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/divisions/{child_id}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"parent_division_id": null}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert!(updated["parent_division_id"].is_null());

    // Delete
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/divisions/{child_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
