use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::{
        payroll_concept::{PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType},
        payroll_history_detail::PayrollHistoryDetail,
    },
    error::{AppError, AppResult},
    server::AppState,
    services::payroll_history_detail::CreatePayrollHistoryDetailParams,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePayrollHistoryDetailRequest {
    pub employee_id: Uuid,
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PayrollHistoryDetailResponse {
    pub id: Uuid,
    pub payroll_history_id: Uuid,
    // Division info
    pub division_id: Uuid,
    pub division_name: String,
    pub division_budget_code: String,
    // Job info
    pub job_id: Uuid,
    pub job_title: String,
    pub job_salary: f64,
    // Employee info
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub employee_last_name: String,
    pub employee_first_name: String,
    #[schema(value_type = String, format = Date)]
    pub employee_hire_date: NaiveDate,
    pub employee_salary: f64,
    pub employee_clasification: String,
    pub employee_hours: i32,
    pub employee_bank_account: String,
    // Bank info
    pub bank_id: Uuid,
    pub bank_name: String,
    // Payroll concept info
    pub payroll_concept_id: Uuid,
    pub payroll_concept_code: String,
    pub payroll_concept_name: String,
    pub payroll_concept_type: PayrollConceptType,
    pub payroll_concept_scope: PayrollConceptScope,
    pub payroll_concept_period: PayrollConceptPeriod,
    // Amount
    pub amount: f64,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryDetailCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryDetailPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
    pub detail_id: Uuid,
}

impl From<PayrollHistoryDetail> for PayrollHistoryDetailResponse {
    fn from(value: PayrollHistoryDetail) -> Self {
        Self {
            id: value.id,
            payroll_history_id: value.payroll_history_id,
            division_id: value.division_id,
            division_name: value.division_name,
            division_budget_code: value.division_budget_code,
            job_id: value.job_id,
            job_title: value.job_title,
            job_salary: value.job_salary,
            employee_id: value.employee_id,
            employee_id_number: value.employee_id_number,
            employee_last_name: value.employee_last_name,
            employee_first_name: value.employee_first_name,
            employee_hire_date: value.employee_hire_date,
            employee_salary: value.employee_salary,
            employee_clasification: value.employee_clasification,
            employee_hours: value.employee_hours,
            employee_bank_account: value.employee_bank_account,
            bank_id: value.bank_id,
            bank_name: value.bank_name,
            payroll_concept_id: value.payroll_concept_id,
            payroll_concept_code: value.payroll_concept_code,
            payroll_concept_name: value.payroll_concept_name,
            payroll_concept_type: value.payroll_concept_type,
            payroll_concept_scope: value.payroll_concept_scope,
            payroll_concept_period: value.payroll_concept_period,
            amount: value.amount,
        }
    }
}

impl From<CreatePayrollHistoryDetailRequest> for CreatePayrollHistoryDetailParams {
    fn from(req: CreatePayrollHistoryDetailRequest) -> Self {
        Self {
            employee_id: req.employee_id,
            payroll_concept_id: req.payroll_concept_id,
            amount: req.amount,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details",
    params(PayrollHistoryDetailCollectionPathParams),
    request_body = CreatePayrollHistoryDetailRequest,
    responses(
        (status = 201, description = "Payroll history detail created", body = PayrollHistoryDetailResponse)
    ),
    tag = "Payroll History Details",
    operation_id = "create_payroll_history_detail"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryDetailCollectionPathParams>,
    Json(payload): Json<CreatePayrollHistoryDetailRequest>,
) -> AppResult<(StatusCode, Json<PayrollHistoryDetailResponse>)> {
    let detail = state
        .payroll_history_detail_service()
        .create(
            params.organization_id,
            params.payroll_id,
            params.history_id,
            payload.into(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(detail.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details",
    params(PayrollHistoryDetailCollectionPathParams),
    responses(
        (status = 200, description = "List payroll history details", body = [PayrollHistoryDetailResponse])
    ),
    tag = "Payroll History Details",
    operation_id = "list_payroll_history_details"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryDetailCollectionPathParams>,
) -> AppResult<Json<Vec<PayrollHistoryDetailResponse>>> {
    let details = state
        .payroll_history_detail_service()
        .list(params.organization_id, params.payroll_id, params.history_id)
        .await?;
    let response = details
        .into_iter()
        .map(PayrollHistoryDetailResponse::from)
        .collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details/{detail_id}",
    params(PayrollHistoryDetailPathParams),
    responses(
        (status = 200, description = "Get payroll history detail", body = PayrollHistoryDetailResponse),
        (status = 404, description = "Payroll history detail not found")
    ),
    tag = "Payroll History Details",
    operation_id = "get_payroll_history_detail"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryDetailPathParams>,
) -> AppResult<Json<PayrollHistoryDetailResponse>> {
    let detail = state
        .payroll_history_detail_service()
        .get(
            params.organization_id,
            params.payroll_id,
            params.history_id,
            params.detail_id,
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll history detail `{}` not found for history `{}`",
                params.detail_id, params.history_id
            ))
        })?;

    Ok(Json(detail.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/details/{detail_id}",
    params(PayrollHistoryDetailPathParams),
    responses(
        (status = 204, description = "Payroll history detail deleted"),
        (status = 404, description = "Payroll history detail not found")
    ),
    tag = "Payroll History Details",
    operation_id = "delete_payroll_history_detail"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryDetailPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .payroll_history_detail_service()
        .delete(
            params.organization_id,
            params.payroll_id,
            params.history_id,
            params.detail_id,
        )
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "payroll history detail `{}` not found for history `{}`",
            params.detail_id, params.history_id
        )))
    }
}
