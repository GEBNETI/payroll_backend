use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{
    error::AppResult, server::AppState, services::payroll_history_report::EarningsDeductionsReport,
    services::payroll_history_report::PayrollReport,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryReportPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/earnings-deductions",
    params(PayrollHistoryReportPathParams),
    responses(
        (status = 200, description = "Earnings and deductions report", body = EarningsDeductionsReport),
        (status = 404, description = "Payroll history not found")
    ),
    tag = "Payroll History Reports",
    operation_id = "get_earnings_deductions_report"
)]
pub async fn earnings_deductions(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryReportPathParams>,
) -> AppResult<Json<EarningsDeductionsReport>> {
    let report = state
        .payroll_history_report_service()
        .earnings_deductions(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    Ok(Json(report))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/payroll",
    params(PayrollHistoryReportPathParams),
    responses(
        (status = 200, description = "Payroll report", body = PayrollReport),
        (status = 404, description = "Payroll history not found")
    ),
    tag = "Payroll History Reports",
    operation_id = "get_payroll_report"
)]
pub async fn payroll(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryReportPathParams>,
) -> AppResult<Json<PayrollReport>> {
    let report = state
        .payroll_history_report_service()
        .payroll(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    Ok(Json(report))
}
