use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    server::AppState,
    services::payroll_history_report::EarningsDeductionsReport,
    services::payroll_history_report::PayrollReport,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryReportPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PatriaReportQueryParams {
    #[param(value_type = String, format = Date)]
    pub payment_date: NaiveDate,
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

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}/reports/patria",
    params(PayrollHistoryReportPathParams, PatriaReportQueryParams),
    responses(
        (status = 200, description = "Patria payment text file", content_type = "text/plain"),
        (status = 404, description = "Payroll history or payment details not found")
    ),
    tag = "Payroll History Reports",
    operation_id = "get_patria_report"
)]
pub async fn patria(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryReportPathParams>,
    Query(query): Query<PatriaReportQueryParams>,
) -> AppResult<axum::response::Response> {
    let file = state
        .payroll_history_report_service()
        .patria_text_file(
            params.organization_id,
            params.payroll_id,
            params.history_id,
            query.payment_date,
        )
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    let disposition = format!("inline; filename=\"{}\"", file.filename);
    headers.insert(
        header::CONTENT_DISPOSITION,
        disposition
            .parse()
            .map_err(|_| AppError::internal("failed to set Patria filename"))?,
    );

    Ok((headers, file.content).into_response())
}
