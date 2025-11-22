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
                .body(Body::from(json!({"name": "Employees Org"}).to_string()))
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
                .uri(format!("/organizations/{organization_id}/payrolls"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Employees Payroll",
                        "description": "Payroll for employees"
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

async fn create_bank(app: &Router, organization_id: Uuid, name: &str) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/banks"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": name}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload = read_json(response.into_body().collect().await.unwrap().to_bytes());
    Uuid::parse_str(payload["id"].as_str().unwrap()).expect("uuid")
}

async fn create_job(app: &Router, organization_id: Uuid, payroll_id: Uuid, title: &str) -> Uuid {
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
                        "job_title": title,
                        "salary": 50000.0
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
    organization_id: Uuid,
    payroll_id: Uuid,
    name: &str,
) -> Uuid {
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
                        "name": name,
                        "description": format!("{name} division"),
                        "budget_code": format!("BC-{name}")
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

#[tokio::test]
async fn can_create_and_list_employees() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let bank_id = create_bank(&app, organization_id, "Nomina Bank").await;
    let job_id = create_job(&app, organization_id, payroll_id, "Analyst").await;
    let division_id = create_division(&app, organization_id, payroll_id, "Ops").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees"))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "id_number": "123-456",
                    "last_name": "Doe",
                    "first_name": "Jane",
                    "address": "123 Main St",
                    "phone": "555-1111",
                    "place_of_birth": "Townsville",
                    "date_of_birth": "1990-01-01",
                    "nationality": "Exampleland",
                    "marital_status": "Single",
                    "gender": "F",
                    "hire_date": "2020-01-01",
                    "clasification": "Full-time",
                    "job_id": job_id,
                    "bank_id": bank_id,
                    "bank_account": "ACC123",
                    "status": "Active",
                    "hours": 40
                }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(created["division_id"], division_id.to_string());
    assert_eq!(created["job_id"], job_id.to_string());
    assert_eq!(created["bank_id"], bank_id.to_string());
    assert!(created["termination_date"].is_null());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let list = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["last_name"], "Doe");
}

#[tokio::test]
async fn rejects_invalid_references_and_dates() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let other_org = create_organization(&app).await;
    let payroll_a = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_a, "Engineering").await;
    let bank_other_org = create_bank(&app, other_org, "Ghost Bank").await;
    let bank_valid = create_bank(&app, organization_id, "Solid Bank").await;
    let job_in_payroll = create_job(&app, organization_id, payroll_a, "Contractor").await;

    // Bank must belong to organization
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_a}/divisions/{division_id}/employees"))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "id_number": "BADBANK",
                    "last_name": "Bad",
                    "first_name": "Bank",
                    "address": "Unknown",
                    "phone": "555-0000",
                    "place_of_birth": "Nowhere",
                    "date_of_birth": "1991-01-01",
                    "nationality": "None",
                    "marital_status": "Single",
                    "gender": "X",
                    "hire_date": "2022-01-01",
                    "clasification": "Temp",
                    "job_id": job_in_payroll,
                    "bank_id": bank_other_org,
                    "bank_account": "ACC000",
                    "status": "Active",
                    "hours": 10
                }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Leaving date before hire date rejected
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_a}/divisions/{division_id}/employees"))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "id_number": "DATEERR",
                    "last_name": "Time",
                    "first_name": "Traveler",
                    "address": "123 Time Rd",
                    "phone": "555-9999",
                    "place_of_birth": "Clocktown",
                    "date_of_birth": "1991-01-01",
                    "nationality": "Timeland",
                    "marital_status": "Single",
                    "gender": "M",
                    "hire_date": "2022-01-02",
                    "termination_date": "2022-01-01",
                    "clasification": "Full-time",
                    "job_id": job_in_payroll,
                    "bank_id": bank_valid,
                    "bank_account": "ACC999",
                    "status": "Inactive",
                    "hours": 20
                }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn can_update_and_delete_employee() {
    let app = support::test_router();
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let bank_id = create_bank(&app, organization_id, "Hiring Bank").await;
    let job_id = create_job(&app, organization_id, payroll_id, "Assistant").await;
    let division_id = create_division(&app, organization_id, payroll_id, "Support").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees"))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "id_number": "ABC123",
                    "last_name": "Smith",
                    "first_name": "Alex",
                    "address": "456 Side St",
                    "phone": "555-2222",
                    "place_of_birth": "Ville",
                    "date_of_birth": "1985-05-05",
                    "nationality": "Testland",
                    "marital_status": "Married",
                    "gender": "F",
                    "hire_date": "2021-06-01",
                    "clasification": "Part-time",
                    "job_id": job_id,
                    "bank_id": bank_id,
                    "bank_account": "ACCT-456",
                    "status": "Active",
                    "hours": 25
                }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let created = read_json(response.into_body().collect().await.unwrap().to_bytes());
    let employee_id = created["id"].as_str().unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "hours": 30,
                    "status": "On Leave",
                    "termination_date": null
                }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let updated = read_json(response.into_body().collect().await.unwrap().to_bytes());
    assert_eq!(updated["hours"], 30);
    assert_eq!(updated["status"], "On Leave");
    assert!(updated["termination_date"].is_null());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}"))
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
                .uri(format!("/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
