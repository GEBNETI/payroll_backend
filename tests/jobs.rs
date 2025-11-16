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
                .body(Body::from(json!({"name": "Jobs Org"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    Uuid::parse_str(body["id"].as_str().unwrap()).expect("uuid")
}

async fn create_payroll(app: &Router, organization_id: Uuid) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "August Payroll",
                        "description": "Jobs payroll",
                    })
                    .to_string(),
                ))
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
async fn can_create_and_list_jobs() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "job_title": "Software Engineer",
                        "salary": 100_000.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["job_title"], "Software Engineer");
    assert_eq!(created["salary"], 100000.0);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let list = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["payroll_id"], payroll_id.to_string());
}

#[tokio::test]
async fn rejects_invalid_payroll_reference() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    // Create a payroll for reference but use an unrelated payroll id.
    let _ = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{}/jobs",
                    Uuid::new_v4()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "job_title": "Ghost Job",
                        "salary": 50_000.0
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
async fn can_update_and_delete_job() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "job_title": "Designer",
                        "salary": 80_000.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let job_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "job_title": "Senior Designer",
                        "salary": 90_000.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["job_title"], "Senior Designer");
    assert_eq!(updated["salary"], 90000.0);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}"
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
                    "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
