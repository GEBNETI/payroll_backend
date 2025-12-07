use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::employee_payroll_concept::EmployeePayrollConcept,
    error::{AppError, AppResult},
    server::AppState,
    services::employee_payroll_concept::{
        CreateEmployeePayrollConceptParams, UpdateEmployeePayrollConceptParams,
    },
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEmployeePayrollConceptRequest {
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEmployeePayrollConceptRequest {
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmployeePayrollConceptResponse {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct EmployeePayrollConceptCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub division_id: Uuid,
    pub employee_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct EmployeePayrollConceptPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub division_id: Uuid,
    pub employee_id: Uuid,
    pub assignment_id: Uuid,
}

impl From<EmployeePayrollConcept> for EmployeePayrollConceptResponse {
    fn from(value: EmployeePayrollConcept) -> Self {
        Self {
            id: value.id,
            employee_id: value.employee_id,
            payroll_concept_id: value.payroll_concept_id,
            amount: value.amount,
        }
    }
}

impl From<CreateEmployeePayrollConceptRequest> for CreateEmployeePayrollConceptParams {
    fn from(req: CreateEmployeePayrollConceptRequest) -> Self {
        Self {
            payroll_concept_id: req.payroll_concept_id,
            amount: req.amount,
        }
    }
}

impl From<UpdateEmployeePayrollConceptRequest> for UpdateEmployeePayrollConceptParams {
    fn from(req: UpdateEmployeePayrollConceptRequest) -> Self {
        Self { amount: req.amount }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts",
    params(EmployeePayrollConceptCollectionPathParams),
    request_body = CreateEmployeePayrollConceptRequest,
    responses(
        (status = 201, description = "Payroll concept assigned to employee", body = EmployeePayrollConceptResponse)
    ),
    tag = "Employee Payroll Concepts",
    operation_id = "create_employee_payroll_concept"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<EmployeePayrollConceptCollectionPathParams>,
    Json(payload): Json<CreateEmployeePayrollConceptRequest>,
) -> AppResult<(StatusCode, Json<EmployeePayrollConceptResponse>)> {
    let assignment = state
        .employee_payroll_concept_service()
        .create(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
            payload.into(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(assignment.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts",
    params(EmployeePayrollConceptCollectionPathParams),
    responses(
        (status = 200, description = "List employee payroll concepts", body = [EmployeePayrollConceptResponse])
    ),
    tag = "Employee Payroll Concepts",
    operation_id = "list_employee_payroll_concepts"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<EmployeePayrollConceptCollectionPathParams>,
) -> AppResult<Json<Vec<EmployeePayrollConceptResponse>>> {
    let assignments = state
        .employee_payroll_concept_service()
        .list(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
        )
        .await?;

    let response = assignments
        .into_iter()
        .map(EmployeePayrollConceptResponse::from)
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}",
    params(EmployeePayrollConceptPathParams),
    responses(
        (status = 200, description = "Get employee payroll concept", body = EmployeePayrollConceptResponse),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Employee Payroll Concepts",
    operation_id = "get_employee_payroll_concept"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<EmployeePayrollConceptPathParams>,
) -> AppResult<Json<EmployeePayrollConceptResponse>> {
    let assignment = state
        .employee_payroll_concept_service()
        .get(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
            params.assignment_id,
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "assignment `{}` not found for employee `{}` in payroll `{}`",
                params.assignment_id, params.employee_id, params.payroll_id
            ))
        })?;

    Ok(Json(assignment.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}",
    params(EmployeePayrollConceptPathParams),
    request_body = UpdateEmployeePayrollConceptRequest,
    responses(
        (status = 200, description = "Employee payroll concept updated", body = EmployeePayrollConceptResponse),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Employee Payroll Concepts",
    operation_id = "update_employee_payroll_concept"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<EmployeePayrollConceptPathParams>,
    Json(payload): Json<UpdateEmployeePayrollConceptRequest>,
) -> AppResult<Json<EmployeePayrollConceptResponse>> {
    let assignment = state
        .employee_payroll_concept_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
            params.assignment_id,
            payload.into(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "assignment `{}` not found for employee `{}` in payroll `{}`",
                params.assignment_id, params.employee_id, params.payroll_id
            ))
        })?;

    Ok(Json(assignment.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}/concepts/{assignment_id}",
    params(EmployeePayrollConceptPathParams),
    responses(
        (status = 204, description = "Employee payroll concept removed"),
        (status = 404, description = "Assignment not found")
    ),
    tag = "Employee Payroll Concepts",
    operation_id = "delete_employee_payroll_concept"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<EmployeePayrollConceptPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .employee_payroll_concept_service()
        .delete(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
            params.assignment_id,
        )
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "assignment `{}` not found for employee `{}` in payroll `{}`",
            params.assignment_id, params.employee_id, params.payroll_id
        )))
    }
}
