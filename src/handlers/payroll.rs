use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::payroll::Payroll,
    error::{AppError, AppResult},
    server::AppState,
    services::payroll::{CreatePayrollParams, UpdatePayrollParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePayrollRequest {
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePayrollRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PayrollResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollPathParams {
    pub id: Uuid,
}

impl From<Payroll> for PayrollResponse {
    fn from(value: Payroll) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            organization_id: value.organization_id,
        }
    }
}

impl CreatePayrollRequest {
    fn into_params(self) -> CreatePayrollParams {
        CreatePayrollParams {
            name: self.name,
            description: self.description,
            organization_id: self.organization_id,
        }
    }
}

impl UpdatePayrollRequest {
    fn into_params(self) -> UpdatePayrollParams {
        UpdatePayrollParams {
            name: self.name,
            description: self.description,
            organization_id: self.organization_id,
        }
    }
}

#[utoipa::path(
    post,
    path = "/payrolls",
    request_body = CreatePayrollRequest,
    responses(
        (status = 201, description = "Payroll created", body = PayrollResponse)
    ),
    tag = "Payrolls"
)]
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreatePayrollRequest>,
) -> AppResult<(StatusCode, Json<PayrollResponse>)> {
    let payroll = state
        .payroll_service()
        .create(payload.into_params())
        .await?;

    Ok((StatusCode::CREATED, Json(payroll.into())))
}

#[utoipa::path(
    get,
    path = "/payrolls",
    responses(
        (status = 200, description = "List payrolls", body = [PayrollResponse])
    ),
    tag = "Payrolls"
)]
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<PayrollResponse>>> {
    let payrolls = state.payroll_service().list().await?;
    let response = payrolls.into_iter().map(PayrollResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/payrolls/{id}",
    params(PayrollPathParams),
    responses(
        (status = 200, description = "Get payroll", body = PayrollResponse),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
) -> AppResult<Json<PayrollResponse>> {
    let id = params.id;
    let payroll = state
        .payroll_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("payroll `{id}` not found")))?;

    Ok(Json(payroll.into()))
}

#[utoipa::path(
    put,
    path = "/payrolls/{id}",
    params(PayrollPathParams),
    request_body = UpdatePayrollRequest,
    responses(
        (status = 200, description = "Payroll updated", body = PayrollResponse),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
    Json(payload): Json<UpdatePayrollRequest>,
) -> AppResult<Json<PayrollResponse>> {
    let id = params.id;
    let payroll = state
        .payroll_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("payroll `{id}` not found")))?;

    Ok(Json(payroll.into()))
}

#[utoipa::path(
    delete,
    path = "/payrolls/{id}",
    params(PayrollPathParams),
    responses(
        (status = 204, description = "Payroll deleted"),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
) -> AppResult<StatusCode> {
    let id = params.id;
    let removed = state.payroll_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("payroll `{id}` not found")))
    }
}
