use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::payroll_history::{PayrollHistory, PayrollHistoryStatus},
    error::{AppError, AppResult},
    server::AppState,
    services::payroll_history::{CreatePayrollHistoryParams, UpdatePayrollHistoryParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePayrollHistoryRequest {
    pub title: String,
    pub period: String,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub end_date: NaiveDate,
    pub status: PayrollHistoryStatus,
    pub total_employees: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePayrollHistoryRequest {
    pub title: Option<String>,
    pub period: Option<String>,
    #[schema(value_type = Option<String>, format = Date)]
    pub start_date: Option<NaiveDate>,
    #[schema(value_type = Option<String>, format = Date)]
    pub end_date: Option<NaiveDate>,
    pub status: Option<PayrollHistoryStatus>,
    pub total_employees: Option<i32>,
    pub total_earnings: Option<f64>,
    pub total_deductions: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PayrollHistoryResponse {
    pub id: Uuid,
    pub title: String,
    pub period: String,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub end_date: NaiveDate,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub payroll_id: Uuid,
    pub payroll_name: String,
    pub status: PayrollHistoryStatus,
    pub total_employees: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollHistoryPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub history_id: Uuid,
}

impl From<PayrollHistory> for PayrollHistoryResponse {
    fn from(value: PayrollHistory) -> Self {
        Self {
            id: value.id,
            title: value.title,
            period: value.period,
            start_date: value.start_date,
            end_date: value.end_date,
            created_at: value.created_at,
            organization_id: value.organization_id,
            organization_name: value.organization_name,
            payroll_id: value.payroll_id,
            payroll_name: value.payroll_name,
            status: value.status,
            total_employees: value.total_employees,
            total_earnings: value.total_earnings,
            total_deductions: value.total_deductions,
        }
    }
}

impl From<CreatePayrollHistoryRequest> for CreatePayrollHistoryParams {
    fn from(req: CreatePayrollHistoryRequest) -> Self {
        Self {
            title: req.title,
            period: req.period,
            start_date: req.start_date,
            end_date: req.end_date,
            status: req.status,
            total_employees: req.total_employees,
            total_earnings: req.total_earnings,
            total_deductions: req.total_deductions,
        }
    }
}

impl From<UpdatePayrollHistoryRequest> for UpdatePayrollHistoryParams {
    fn from(req: UpdatePayrollHistoryRequest) -> Self {
        Self {
            title: req.title,
            period: req.period,
            start_date: req.start_date,
            end_date: req.end_date,
            status: req.status,
            total_employees: req.total_employees,
            total_earnings: req.total_earnings,
            total_deductions: req.total_deductions,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history",
    params(PayrollHistoryCollectionPathParams),
    request_body = CreatePayrollHistoryRequest,
    responses(
        (status = 201, description = "Payroll history created", body = PayrollHistoryResponse)
    ),
    tag = "Payroll History",
    operation_id = "create_payroll_history"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryCollectionPathParams>,
    Json(payload): Json<CreatePayrollHistoryRequest>,
) -> AppResult<(StatusCode, Json<PayrollHistoryResponse>)> {
    let history = state
        .payroll_history_service()
        .create(params.organization_id, params.payroll_id, payload.into())
        .await?;

    Ok((StatusCode::CREATED, Json(history.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history",
    params(PayrollHistoryCollectionPathParams),
    responses(
        (status = 200, description = "List payroll history", body = [PayrollHistoryResponse])
    ),
    tag = "Payroll History",
    operation_id = "list_payroll_history"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryCollectionPathParams>,
) -> AppResult<Json<Vec<PayrollHistoryResponse>>> {
    let histories = state
        .payroll_history_service()
        .list(params.organization_id, params.payroll_id)
        .await?;
    let response = histories
        .into_iter()
        .map(PayrollHistoryResponse::from)
        .collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}",
    params(PayrollHistoryPathParams),
    responses(
        (status = 200, description = "Get payroll history", body = PayrollHistoryResponse),
        (status = 404, description = "Payroll history not found")
    ),
    tag = "Payroll History",
    operation_id = "get_payroll_history"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryPathParams>,
) -> AppResult<Json<PayrollHistoryResponse>> {
    let history = state
        .payroll_history_service()
        .get(params.organization_id, params.payroll_id, params.history_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll history `{}` not found for payroll `{}`",
                params.history_id, params.payroll_id
            ))
        })?;

    Ok(Json(history.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}",
    params(PayrollHistoryPathParams),
    request_body = UpdatePayrollHistoryRequest,
    responses(
        (status = 200, description = "Payroll history updated", body = PayrollHistoryResponse),
        (status = 404, description = "Payroll history not found")
    ),
    tag = "Payroll History",
    operation_id = "update_payroll_history"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryPathParams>,
    Json(payload): Json<UpdatePayrollHistoryRequest>,
) -> AppResult<Json<PayrollHistoryResponse>> {
    let history = state
        .payroll_history_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.history_id,
            payload.into(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll history `{}` not found for payroll `{}`",
                params.history_id, params.payroll_id
            ))
        })?;

    Ok(Json(history.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/history/{history_id}",
    params(PayrollHistoryPathParams),
    responses(
        (status = 204, description = "Payroll history deleted"),
        (status = 404, description = "Payroll history not found")
    ),
    tag = "Payroll History",
    operation_id = "delete_payroll_history"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollHistoryPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .payroll_history_service()
        .delete(params.organization_id, params.payroll_id, params.history_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "payroll history `{}` not found for payroll `{}`",
            params.history_id, params.payroll_id
        )))
    }
}
