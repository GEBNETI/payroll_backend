use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::payroll::Payroll,
    error::{AppError, AppResult},
    server::AppState,
    services::payroll::{CreatePayrollParams, UpdatePayrollParams},
};

#[derive(Debug, Deserialize)]
pub struct CreatePayrollRequest {
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayrollRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PayrollResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
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

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<PayrollResponse>>> {
    let payrolls = state.payroll_service().list().await?;
    let response = payrolls.into_iter().map(PayrollResponse::from).collect();
    Ok(Json(response))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PayrollResponse>> {
    let payroll = state
        .payroll_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("payroll `{id}` not found")))?;

    Ok(Json(payroll.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePayrollRequest>,
) -> AppResult<Json<PayrollResponse>> {
    let payroll = state
        .payroll_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("payroll `{id}` not found")))?;

    Ok(Json(payroll.into()))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let removed = state.payroll_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("payroll `{id}` not found")))
    }
}
