use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::bank::Bank,
    error::{AppError, AppResult},
    server::AppState,
    services::bank::{CreateBankParams, UpdateBankParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBankRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBankRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankResponse {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct OrganizationPathParams {
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct BankPathParams {
    pub organization_id: Uuid,
    pub bank_id: Uuid,
}

impl From<Bank> for BankResponse {
    fn from(value: Bank) -> Self {
        Self {
            id: value.id,
            name: value.name,
            organization_id: value.organization_id,
        }
    }
}

impl CreateBankRequest {
    fn into_params(self) -> CreateBankParams {
        CreateBankParams { name: self.name }
    }
}

impl UpdateBankRequest {
    fn into_params(self) -> UpdateBankParams {
        UpdateBankParams { name: self.name }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/banks",
    params(OrganizationPathParams),
    request_body = CreateBankRequest,
    responses(
        (status = 201, description = "Bank created", body = BankResponse)
    ),
    tag = "Banks",
    operation_id = "create_bank"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
    Json(payload): Json<CreateBankRequest>,
) -> AppResult<(StatusCode, Json<BankResponse>)> {
    let bank = state
        .bank_service()
        .create(params.organization_id, payload.into_params())
        .await?;

    Ok((StatusCode::CREATED, Json(bank.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/banks",
    params(OrganizationPathParams),
    responses(
        (status = 200, description = "List banks", body = [BankResponse])
    ),
    tag = "Banks",
    operation_id = "list_banks"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
) -> AppResult<Json<Vec<BankResponse>>> {
    let banks = state.bank_service().list(params.organization_id).await?;
    let response = banks.into_iter().map(BankResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/banks/{bank_id}",
    params(BankPathParams),
    responses(
        (status = 200, description = "Get bank", body = BankResponse),
        (status = 404, description = "Bank not found")
    ),
    tag = "Banks",
    operation_id = "get_bank"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<BankPathParams>,
) -> AppResult<Json<BankResponse>> {
    let bank = state
        .bank_service()
        .get(params.organization_id, params.bank_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "bank `{}` not found for organization `{}`",
                params.bank_id, params.organization_id
            ))
        })?;

    Ok(Json(bank.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/banks/{bank_id}",
    params(BankPathParams),
    request_body = UpdateBankRequest,
    responses(
        (status = 200, description = "Bank updated", body = BankResponse),
        (status = 404, description = "Bank not found")
    ),
    tag = "Banks",
    operation_id = "update_bank"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<BankPathParams>,
    Json(payload): Json<UpdateBankRequest>,
) -> AppResult<Json<BankResponse>> {
    let bank = state
        .bank_service()
        .update(
            params.organization_id,
            params.bank_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "bank `{}` not found for organization `{}`",
                params.bank_id, params.organization_id
            ))
        })?;

    Ok(Json(bank.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/banks/{bank_id}",
    params(BankPathParams),
    responses(
        (status = 204, description = "Bank deleted"),
        (status = 404, description = "Bank not found")
    ),
    tag = "Banks",
    operation_id = "delete_bank"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<BankPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .bank_service()
        .delete(params.organization_id, params.bank_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "bank `{}` not found for organization `{}`",
            params.bank_id, params.organization_id
        )))
    }
}
