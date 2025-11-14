use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
    server::AppState,
    services::organization::{CreateOrganizationParams, UpdateOrganizationParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct OrganizationPathParams {
    pub id: Uuid,
}

impl From<Organization> for OrganizationResponse {
    fn from(value: Organization) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl CreateOrganizationRequest {
    fn into_params(self) -> CreateOrganizationParams {
        CreateOrganizationParams { name: self.name }
    }
}

impl UpdateOrganizationRequest {
    fn into_params(self) -> UpdateOrganizationParams {
        UpdateOrganizationParams { name: self.name }
    }
}

#[utoipa::path(
    post,
    path = "/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created", body = OrganizationResponse)
    ),
    tag = "Organizations"
)]
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> AppResult<(StatusCode, Json<OrganizationResponse>)> {
    let organization = state
        .organization_service()
        .create(payload.into_params())
        .await?;

    Ok((StatusCode::CREATED, Json(organization.into())))
}

#[utoipa::path(
    get,
    path = "/organizations",
    responses(
        (status = 200, description = "List organizations", body = [OrganizationResponse])
    ),
    tag = "Organizations"
)]
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<OrganizationResponse>>> {
    let organizations = state.organization_service().list().await?;
    let response = organizations
        .into_iter()
        .map(OrganizationResponse::from)
        .collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{id}",
    params(OrganizationPathParams),
    responses(
        (status = 200, description = "Get organization", body = OrganizationResponse),
        (status = 404, description = "Organization not found")
    ),
    tag = "Organizations"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
) -> AppResult<Json<OrganizationResponse>> {
    let id = params.id;
    let organization = state
        .organization_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("organization `{id}` not found")))?;

    Ok(Json(organization.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{id}",
    params(OrganizationPathParams),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 404, description = "Organization not found")
    ),
    tag = "Organizations"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> AppResult<Json<OrganizationResponse>> {
    let id = params.id;
    let organization = state
        .organization_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("organization `{id}` not found")))?;

    Ok(Json(organization.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{id}",
    params(OrganizationPathParams),
    responses(
        (status = 204, description = "Organization deleted"),
        (status = 404, description = "Organization not found")
    ),
    tag = "Organizations"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<OrganizationPathParams>,
) -> AppResult<StatusCode> {
    let id = params.id;
    let removed = state.organization_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "organization `{id}` not found"
        )))
    }
}
