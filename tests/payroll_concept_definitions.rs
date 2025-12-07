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
                .body(Body::from(json!({"name": "Definition Org"}).to_string()))
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
                        "name": "Definition Payroll",
                        "description": "Payroll with definitions",
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

async fn create_concept(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    scope: &str,
) -> Uuid {
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
                        "code": format!("DEF-{scope}"),
                        "name": format!("{scope} Concept"),
                        "type": "earning",
                        "scope": scope,
                        "period": "both",
                        "active": true
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
async fn can_manage_payroll_concept_definition() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let concept_id = create_concept(&app, organization_id, payroll_id, "global").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "formula": "salary * 0.1",
                        "condition": "hours > 0"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["formula"], "salary * 0.1");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "formula": "salary * 0.15",
                        "condition": "hours >= 40"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["formula"], "salary * 0.15");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
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
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_individual_concept_definitions() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let concept_id = create_concept(&app, organization_id, payroll_id, "individual").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "formula": "salary * 0.1",
                        "condition": "hours > 0"
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
async fn rejects_duplicate_definitions() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let concept_id = create_concept(&app, organization_id, payroll_id, "global").await;

    for (index, expected_status) in [StatusCode::CREATED, StatusCode::CONFLICT]
        .into_iter()
        .enumerate()
    {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "formula": format!("salary * 0.{}", index),
                            "condition": "hours > 0"
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), expected_status);
    }
}
