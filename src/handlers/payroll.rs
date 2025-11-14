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
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePayrollRequest {
    pub name: Option<String>,
    pub description: Option<String>,
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
pub struct OrganizationPathParams {
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
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
        }
    }
}

impl UpdatePayrollRequest {
    fn into_params(self) -> UpdatePayrollParams {
        UpdatePayrollParams {
            name: self.name,
            description: self.description,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls",
    params(OrganizationPathParams),
    request_body = CreatePayrollRequest,
    responses(
        (status = 201, description = "Payroll created", body = PayrollResponse)
    ),
    tag = "Payrolls",
    operation_id = "create_payroll"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
    Json(payload): Json<CreatePayrollRequest>,
) -> AppResult<(StatusCode, Json<PayrollResponse>)> {
    let payroll = state
        .payroll_service()
        .create(params.organization_id, payload.into_params())
        .await?;

    Ok((StatusCode::CREATED, Json(payroll.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls",
    params(OrganizationPathParams),
    responses(
        (status = 200, description = "List payrolls", body = [PayrollResponse])
    ),
    tag = "Payrolls",
    operation_id = "list_payrolls"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
) -> AppResult<Json<Vec<PayrollResponse>>> {
    let payrolls = state.payroll_service().list(params.organization_id).await?;
    let response = payrolls.into_iter().map(PayrollResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}",
    params(PayrollPathParams),
    responses(
        (status = 200, description = "Get payroll", body = PayrollResponse),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls",
    operation_id = "get_payroll"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
) -> AppResult<Json<PayrollResponse>> {
    let payroll = state
        .payroll_service()
        .get(params.organization_id, params.payroll_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll `{}` not found for organization `{}`",
                params.payroll_id, params.organization_id
            ))
        })?;

    Ok(Json(payroll.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}",
    params(PayrollPathParams),
    request_body = UpdatePayrollRequest,
    responses(
        (status = 200, description = "Payroll updated", body = PayrollResponse),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls",
    operation_id = "update_payroll"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
    Json(payload): Json<UpdatePayrollRequest>,
) -> AppResult<Json<PayrollResponse>> {
    let payroll = state
        .payroll_service()
        .update(
            params.organization_id,
            params.payroll_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll `{}` not found for organization `{}`",
                params.payroll_id, params.organization_id
            ))
        })?;

    Ok(Json(payroll.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}",
    params(PayrollPathParams),
    responses(
        (status = 204, description = "Payroll deleted"),
        (status = 404, description = "Payroll not found")
    ),
    tag = "Payrolls",
    operation_id = "delete_payroll"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .payroll_service()
        .delete(params.organization_id, params.payroll_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "payroll `{}` not found for organization `{}`",
            params.payroll_id, params.organization_id
        )))
    }
}
