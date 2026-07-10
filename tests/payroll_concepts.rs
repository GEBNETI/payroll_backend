#[path = "support/mod.rs"]
mod support;

use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
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
                .body(Body::from(json!({"name": "Concept Org", "rif": "G000000000"}).to_string()))
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
                        "name": "Concept Payroll",
                        "description": "Payroll with concepts",
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
async fn can_create_and_list_payroll_concepts() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "code": "BASIC",
                        "name": "Base Salary",
                        "type": "earning",
                        "scope": "global",
                        "period": "1",
                        "active": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["code"], "BASIC");
    assert_eq!(created["type"], "earning");
    assert_eq!(created["period"], "1");
    assert!(created["active"].as_bool().unwrap());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts"
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
async fn rejects_invalid_payroll_reference_for_concepts() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let _ = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{}/concepts",
                    Uuid::new_v4()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "code": "INVALID",
                        "name": "Invalid",
                        "type": "deduction",
                        "scope": "individual",
                        "period": "2",
                        "active": true
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
async fn can_update_and_delete_payroll_concept() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "code": "BONUS",
                        "name": "Bonus",
                        "type": "earning",
                        "scope": "individual",
                        "period": "special",
                        "active": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let concept_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "code": "BONUS",
                        "name": "Performance Bonus",
                        "type": "earning",
                        "scope": "global",
                        "period": "both",
                        "active": false
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["name"], "Performance Bonus");
    assert_eq!(updated["period"], "both");
    assert!(!updated["active"].as_bool().unwrap());
    assert_eq!(updated["scope"], "global");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}"
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
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
