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

// Helper functions to create test data

async fn create_organization(app: &Router) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/organizations")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"name": "Calculator Org", "rif": "G000000000"}).to_string(),
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
                        "name": "Calculator Payroll",
                        "description": "Test payroll",
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
                        "name": "Test Division",
                        "description": "Division for calculator tests",
                        "budget_code": "DIV01"
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

async fn create_job(app: &Router, organization_id: Uuid, payroll_id: Uuid, salary: f64) -> Uuid {
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
                        "job_title": "Test Job",
                        "description": "Job for calculator tests",
                        "salary": salary
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
                .body(Body::from(
                    json!({
                        "name": "Test Bank",
                        "code": "BANK01"
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

#[allow(clippy::too_many_arguments)]
async fn create_employee(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    division_id: Uuid,
    job_id: Uuid,
    bank_id: Uuid,
    salary: f64,
    hours: i32,
) -> Uuid {
    let unique_id = Uuid::new_v4();
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
                        "id_number": format!("V{:08}", unique_id.as_u128() % 100_000_000),
                        "first_name": "John",
                        "last_name": "Doe",
                        "address": "123 Test St",
                        "phone": "555-0100",
                        "place_of_birth": "Test City",
                        "date_of_birth": "1990-01-01",
                        "nationality": "US",
                        "marital_status": "single",
                        "gender": "M",
                        "job_id": job_id,
                        "bank_id": bank_id,
                        "bank_account": "01020000000000000001",
                        "salary": salary,
                        "hours": hours,
                        "clasification": "regular",
                        "status": "active",
                        "hire_date": "2024-01-01",
                        "termination_date": null
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
    code: &str,
    concept_type: &str,
    scope: &str,
    period: &str,
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
                        "code": code,
                        "name": format!("Concept {}", code),
                        "type": concept_type,
                        "scope": scope,
                        "period": period,
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

async fn create_employee_payroll_concept(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    division_id: Uuid,
    employee_id: Uuid,
    concept_id: Uuid,
    amount: f64,
) {
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
                        "amount": amount
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_payroll_concept_definition(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    concept_id: Uuid,
    formula: &str,
    condition: &str,
) {
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
                        "formula": formula,
                        "condition": condition
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_payroll_history(
    app: &Router,
    organization_id: Uuid,
    payroll_id: Uuid,
    period: &str,
) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": format!("History {}", period),
                        "period": period,
                        "start_date": "2024-01-01",
                        "end_date": "2024-01-15",
                        "status": "draft",
                        "total_employees": 0,
                        "total_earnings": 0.0,
                        "total_deductions": 0.0
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
async fn can_calculate_with_individual_concepts() {
    let app = support::test_router();

    // Setup: Create organization, payroll, division, job, bank
    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    // Create employee
    let employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        6000.0,
        40,
    )
    .await;

    // Create individual concept
    let bonus_concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BONUS",
        "earning",
        "individual",
        "1",
    )
    .await;

    // Assign concept to employee with specific amount
    create_employee_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        division_id,
        employee_id,
        bonus_concept_id,
        1500.0,
    )
    .await;

    // Create payroll history
    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate payroll
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(result["total_employees"], 1);
    assert_eq!(result["total_details_created"], 1);
    assert_eq!(result["total_earnings"], 1500.0);
    assert_eq!(result["total_deductions"], 0.0);
    assert!(result["warnings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn can_calculate_with_global_concepts_and_formulas() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    // Create employee with salary 3000
    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create global concept with formula
    let tax_concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "TAX",
        "deduction",
        "global",
        "1",
    )
    .await;

    // Define formula: 10% of salary
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        tax_concept_id,
        "salary * 0.1",
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(result["total_employees"], 1);
    assert_eq!(result["total_details_created"], 1);
    assert_eq!(result["total_earnings"], 0.0);
    assert_eq!(result["total_deductions"], 300.0); // 3000 * 0.1 = 300
}

#[tokio::test]
async fn filters_concepts_by_period() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        2000.0,
        40,
    )
    .await;

    // Create concepts with different periods
    let period1_concept = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "P1",
        "earning",
        "global",
        "1",
    )
    .await;

    let period2_concept = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "P2",
        "earning",
        "global",
        "2",
    )
    .await;

    let both_concept = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BOTH",
        "earning",
        "global",
        "both",
    )
    .await;

    // Create definitions for all concepts
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        period1_concept,
        "100",
        "true",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        period2_concept,
        "200",
        "true",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        both_concept,
        "300",
        "true",
    )
    .await;

    // Create history for period "1"
    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should include P1 (100) and BOTH (300), but not P2 (200)
    assert_eq!(result["total_employees"], 1);
    assert_eq!(result["total_details_created"], 2);
    assert_eq!(result["total_earnings"], 400.0); // 100 + 300
}

#[tokio::test]
async fn can_clean_payroll_history_details() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        2000.0,
        40,
    )
    .await;

    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "SALARY",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary",
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate first
    let calc_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(calc_response.status(), StatusCode::OK);

    // Clean
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/clean"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(result["total_details_deleted"], 1);
}

#[tokio::test]
async fn can_recalculate_payroll() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        4000.0,
        40,
    )
    .await;

    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BASE",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary * 0.5",
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate first time
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    // Recalculate (should clean and recalculate)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/recalculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(result["total_employees"], 1);
    assert_eq!(result["total_details_created"], 1);
    assert_eq!(result["total_earnings"], 2000.0); // 4000 * 0.5
}

#[tokio::test]
async fn handles_empty_payroll() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate with no employees
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    assert_eq!(result["total_employees"], 0);
    assert_eq!(result["total_details_created"], 0);
    assert_eq!(result["total_earnings"], 0.0);
    assert_eq!(result["total_deductions"], 0.0);
}

#[tokio::test]
async fn skips_employees_when_condition_is_false() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    // Create two employees with different salaries
    let _employee1 = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        1500.0, // Below threshold
        40,
    )
    .await;

    let _employee2 = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        2500.0, // Above threshold
        40,
    )
    .await;

    // Create concept with condition
    let bonus_concept = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BONUS",
        "earning",
        "global",
        "1",
    )
    .await;

    // Only apply bonus if salary > 2000
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        bonus_concept,
        "500",
        "salary > 2000",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Only employee2 should get the bonus
    assert_eq!(result["total_employees"], 1); // Only one employee has details
    assert_eq!(result["total_details_created"], 1);
    assert_eq!(result["total_earnings"], 500.0);
}

// ================== Edge Case Tests ==================

#[tokio::test]
async fn rejects_calculation_for_non_draft_history() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;

    // Create a history with draft status first
    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Update to finalized status
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "status": "finalized"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    // Try to calculate - should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    // Should return 422 UNPROCESSABLE_ENTITY (validation error)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn detects_self_referencing_formula() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept that references itself
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "SELFREF",
        "earning",
        "global",
        "1",
    )
    .await;

    // Create definition that references its own code
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "SELFREF + 100", // Self-reference!
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should fail with self-reference error
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    // 422 UNPROCESSABLE_ENTITY for validation errors
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn detects_circular_dependency_between_concepts() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create two concepts that reference each other (A -> B -> A)
    let concept_a = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "CONCEPT_A",
        "earning",
        "global",
        "1",
    )
    .await;

    let concept_b = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "CONCEPT_B",
        "earning",
        "global",
        "1",
    )
    .await;

    // A depends on B
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_a,
        "CONCEPT_B + 100",
        "true",
    )
    .await;

    // B depends on A - creates a cycle!
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_b,
        "CONCEPT_A + 200",
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should fail with cycle error
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    // 422 UNPROCESSABLE_ENTITY for validation errors
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn warns_on_invalid_formula_syntax() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept with invalid formula syntax
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "INVALID",
        "earning",
        "global",
        "1",
    )
    .await;

    // Create definition with truly invalid syntax (unclosed parenthesis)
    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary * (100 + 50", // Missing closing parenthesis - truly invalid syntax
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but with warnings and skip the invalid concept
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warnings about invalid formula
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("INVALID") && s.contains("invalid formula syntax")
        }),
        "Expected warning about invalid formula syntax, got: {:?}",
        warnings
    );

    // No details should be created for the invalid concept
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn warns_on_undefined_variable_reference() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept that references an undefined variable
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "USES_UNDEFINED",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary + nonexistent_var", // nonexistent_var doesn't exist
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but with warnings
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warnings about undefined variable
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("USES_UNDEFINED") && s.contains("undefined variable")
        }),
        "Expected warning about undefined variable, got: {:?}",
        warnings
    );
}

#[tokio::test]
async fn warns_on_base_variable_collision() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept with code "salary" which shadows the base variable
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "salary", // This shadows the base context variable!
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "1000", // Fixed value
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but with warnings about shadowing
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warnings about shadowing base variable
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("salary") && s.contains("shadows")
        }),
        "Expected warning about shadowing base variable, got: {:?}",
        warnings
    );
}

#[tokio::test]
async fn warns_on_disallowed_regex_function() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept with regex_matches in formula (should be rejected)
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "REGEX_TEST",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "if(regex_matches(classification, \".*\"), 100, 0)", // Uses regex
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but skip the concept with warning
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about disallowed function
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("REGEX_TEST")
                && s.contains("disallowed functions")
                && s.contains("regex_matches")
        }),
        "Expected warning about disallowed regex function, got: {:?}",
        warnings
    );

    // No details should be created
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn warns_on_negative_employee_balance() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        1000.0, // Low salary
        40,
    )
    .await;

    // Create an earning concept
    let earning_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "SALARY_PAY",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        earning_id,
        "500", // 500 earnings
        "true",
    )
    .await;

    // Create a deduction concept that exceeds earnings
    let deduction_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BIG_TAX",
        "deduction",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        deduction_id,
        "1000", // 1000 deductions > 500 earnings
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about negative balance
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("negative balance") && s.contains("deductions") && s.contains("exceed")
        }),
        "Expected warning about negative balance, got: {:?}",
        warnings
    );

    // Details should still be created
    assert_eq!(result["total_details_created"], 2);
    assert_eq!(result["total_earnings"], 500.0);
    assert_eq!(result["total_deductions"], 1000.0);
}

#[tokio::test]
async fn warns_when_employee_skipped_by_condition() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    // Create employee with low salary (will be skipped by condition)
    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        500.0, // Low salary - won't meet condition
        40,
    )
    .await;

    // Create a concept with condition that requires high salary
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "HIGH_EARNER_BONUS",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "1000",
        "salary > 1000", // Condition: only for employees with salary > 1000
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about skipped employee
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("skipped")
                && s.contains("HIGH_EARNER_BONUS")
                && s.contains("condition evaluated to false")
        }),
        "Expected warning about employee skipped by condition, got: {:?}",
        warnings
    );

    // No details should be created (employee was skipped)
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn warns_on_invalid_condition_syntax() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept with invalid condition syntax
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "BAD_COND",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "1000",
        "salary > (100 + ", // Invalid syntax - unclosed parenthesis
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but skip the concept with warning
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about invalid condition syntax
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("BAD_COND") && s.contains("invalid condition syntax")
        }),
        "Expected warning about invalid condition syntax, got: {:?}",
        warnings
    );

    // No details should be created
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn warns_on_division_by_zero() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept that divides by zero
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "DIV_ZERO",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary / 0", // Division by zero - produces infinity
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but skip the concept with warning
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about non-finite result
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("DIV_ZERO") && (s.contains("non-finite") || s.contains("infinity"))
        }),
        "Expected warning about division by zero / infinity, got: {:?}",
        warnings
    );

    // No details should be created (formula produced non-finite result)
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn warns_on_disallowed_function_in_condition() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let _employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create a concept with random() in condition (should be rejected)
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "RANDOM_COND",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "1000",
        "random() > 0.5", // Uses random in condition - disallowed
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Calculate - should succeed but skip the concept with warning
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should have warning about disallowed function in condition
    let warnings = result["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| {
            let s = w.as_str().unwrap_or("");
            s.contains("RANDOM_COND") && s.contains("disallowed functions") && s.contains("random")
        }),
        "Expected warning about disallowed function in condition, got: {:?}",
        warnings
    );

    // No details should be created
    assert_eq!(result["total_details_created"], 0);
}

#[tokio::test]
async fn allows_variable_names_containing_disallowed_substrings() {
    let app = support::test_router();

    let organization_id = create_organization(&app).await;
    let payroll_id = create_payroll(&app, organization_id).await;
    let division_id = create_division(&app, organization_id, payroll_id).await;
    let job_id = create_job(&app, organization_id, payroll_id, 5000.0).await;
    let bank_id = create_bank(&app, organization_id).await;

    let employee_id = create_employee(
        &app,
        organization_id,
        payroll_id,
        division_id,
        job_id,
        bank_id,
        3000.0,
        40,
    )
    .await;

    // Create an individual concept with "random" in the name (should NOT be blocked)
    let random_bonus_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "random_bonus", // Contains "random" but is a variable name, not a function call
        "earning",
        "individual",
        "1",
    )
    .await;

    // Create a global concept that uses the random_bonus variable
    let concept_id = create_payroll_concept(
        &app,
        organization_id,
        payroll_id,
        "TOTAL_PAY",
        "earning",
        "global",
        "1",
    )
    .await;

    create_payroll_concept_definition(
        &app,
        organization_id,
        payroll_id,
        concept_id,
        "salary + random_bonus", // Uses variable with "random" in name - should be allowed
        "true",
    )
    .await;

    let history_id = create_payroll_history(&app, organization_id, payroll_id, "1").await;

    // Assign the individual concept to the employee
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
                        "payroll_concept_id": random_bonus_id,
                        "amount": 500.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    // Calculate
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let result = read_json(response.into_body().collect().await.unwrap().to_bytes());

    // Should NOT have warning about disallowed function
    let warnings = result["warnings"].as_array().expect("warnings array");
    let has_disallowed_warning = warnings.iter().any(|w| {
        let s = w.as_str().unwrap_or("");
        s.contains("disallowed functions")
    });
    assert!(
        !has_disallowed_warning,
        "Should NOT warn about 'random' in variable name 'random_bonus', got: {:?}",
        warnings
    );

    // Details should be created (2: individual + global)
    assert_eq!(result["total_details_created"], 2);
    // 500 (random_bonus) + 3500 (salary + random_bonus = 3000 + 500)
    assert_eq!(result["total_earnings"], 4000.0);
}
