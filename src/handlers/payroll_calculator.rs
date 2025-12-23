use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    error::AppResult,
    server::AppState,
    services::payroll_calculator::{CalculationResult, CleanResult},
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollCalculatorPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CalculationResponse {
    pub total_employees: i32,
    pub total_details_created: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CleanResponse {
    pub total_details_deleted: usize,
}

impl From<CalculationResult> for CalculationResponse {
    fn from(result: CalculationResult) -> Self {
        Self {
            total_employees: result.total_employees,
            total_details_created: result.total_details_created,
            total_earnings: result.total_earnings,
            total_deductions: result.total_deductions,
            warnings: result.warnings,
        }
    }
}

impl From<CleanResult> for CleanResponse {
    fn from(result: CleanResult) -> Self {
        Self {
            total_details_deleted: result.total_details_deleted,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/calculate",
    params(PayrollCalculatorPathParams),
    responses(
        (status = 200, description = "Payroll calculated successfully", body = CalculationResponse)
    ),
    tag = "Payroll Calculator",
    operation_id = "calculate_payroll"
)]
pub async fn calculate(
    State(state): State<AppState>,
    Path(params): Path<PayrollCalculatorPathParams>,
) -> AppResult<Json<CalculationResponse>> {
    let result = state
        .payroll_calculator_service()
        .calculate(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/clean",
    params(PayrollCalculatorPathParams),
    responses(
        (status = 200, description = "Payroll history details cleaned successfully", body = CleanResponse)
    ),
    tag = "Payroll Calculator",
    operation_id = "clean_payroll_history"
)]
pub async fn clean(
    State(state): State<AppState>,
    Path(params): Path<PayrollCalculatorPathParams>,
) -> AppResult<Json<CleanResponse>> {
    let result = state
        .payroll_calculator_service()
        .clean(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/recalculate",
    params(PayrollCalculatorPathParams),
    responses(
        (status = 200, description = "Payroll recalculated successfully", body = CalculationResponse)
    ),
    tag = "Payroll Calculator",
    operation_id = "recalculate_payroll"
)]
pub async fn recalculate(
    State(state): State<AppState>,
    Path(params): Path<PayrollCalculatorPathParams>,
) -> AppResult<Json<CalculationResponse>> {
    let result = state
        .payroll_calculator_service()
        .recalculate(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    Ok(Json(result.into()))
}
