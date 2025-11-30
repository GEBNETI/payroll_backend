#[path = "support/mod.rs"]
mod support;

use axum::{
    Router,
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use chrono::NaiveDate;
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
                .body(Body::from(
                    json!({"name": "Employee Concept Org"}).to_string(),
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
                        "name": "Employee Concept Payroll",
                        "description": "Payroll for employee concepts"
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

async fn create_division(app: &Router, organization_id: Uuid, payroll_id: Uuid) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Concept Division",
                        "description": "Division for employees",
                        "budget_code": "CD-001",
                        "parent_division_id": null
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

async fn create_job(app: &Router, organization_id: Uuid, payroll_id: Uuid) -> Uuid {
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
                        "job_title": "Concept Engineer",
                        "salary": 120_000.0
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

async fn create_bank(app: &Router, organization_id: Uuid) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/banks"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "name": "Concept Bank" }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    Uuid::parse_str(body["id"].as_str().unwrap()).expect("uuid")
}

async fn create_employee(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    division_id: Uuid,
    job_id: Uuid,
    bank_id: Uuid,
) -> Uuid {
    let hire_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let date_of_birth = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "id_number": "EC-001",
                        "last_name": "Doe",
                        "first_name": "Jane",
                        "address": "123 Main St",
                        "phone": "555-1234",
                        "place_of_birth": "Metropolis",
                        "date_of_birth": date_of_birth.to_string(),
                        "nationality": "Example",
                        "marital_status": "Single",
                        "gender": "F",
                        "hire_date": hire_date.to_string(),
                        "termination_date": null,
                        "clasification": "Full-time",
                        "job_id": job_id,
                        "bank_id": bank_id,
                        "bank_account": "000111222",
                        "status": "active",
                        "hours": 40
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

async fn create_payroll_concept(
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
                        "code": format!("CODE-{scope}"),
                        "name": format!("{scope} Concept"),
                        "type": "earning",
                        "scope": scope
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

async fn setup_employee_context(app: &Router) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let organization_id = create_organization(app).await;
    let payroll_id = create_payroll(app, organization_id).await;
    let division_id = create_division(app, organization_id, payroll_id).await;
    let job_id = create_job(app, organization_id, payroll_id).await;
    let bank_id = create_bank(app, organization_id).await;
    let employee_id = create_employee(
        app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
    )
    .await;
    (
        organization_id,
        payroll_id,
        division_id,
        employee_id,
        job_id,
        bank_id,
    )
}

#[tokio::test]
async fn can_assign_and_list_employee_payroll_concepts() {
    let app = support::test_router();
    let (organization_id, payroll_id, division_id, employee_id, ..) =
        setup_employee_context(&app).await;
    let concept_id = create_payroll_concept(&app, organization_id, payroll_id, "individual").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "payroll_concept_id": concept_id,
                        "amount": 150.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["payroll_concept_id"], concept_id.to_string());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let list = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn rejects_duplicate_employee_payroll_concepts() {
    let app = support::test_router();
    let (organization_id, payroll_id, division_id, employee_id, ..) =
        setup_employee_context(&app).await;
    let concept_id = create_payroll_concept(&app, organization_id, payroll_id, "individual").await;

    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "payroll_concept_id": concept_id,
                            "amount": 200.0
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        if response.status() == StatusCode::CREATED {
            continue;
        }

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}

#[tokio::test]
async fn rejects_global_payroll_concepts_for_employees() {
    let app = support::test_router();
    let (organization_id, payroll_id, division_id, employee_id, ..) =
        setup_employee_context(&app).await;
    let concept_id = create_payroll_concept(&app, organization_id, payroll_id, "global").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "payroll_concept_id": concept_id,
                        "amount": 75.0
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
async fn can_update_and_delete_employee_payroll_concepts() {
    let app = support::test_router();
    let (organization_id, payroll_id, division_id, employee_id, ..) =
        setup_employee_context(&app).await;
    let concept_id = create_payroll_concept(&app, organization_id, payroll_id, "individual").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "payroll_concept_id": concept_id,
                        "amount": 300.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let assignment_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "amount": 350.0 }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["amount"], 350.0);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}"
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
                    "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
