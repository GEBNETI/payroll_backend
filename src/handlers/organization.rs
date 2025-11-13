use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
    server::AppState,
    services::organization::{CreateOrganizationParams, UpdateOrganizationParams},
};

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
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

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<OrganizationResponse>>> {
    let organizations = state.organization_service().list().await?;
    let response = organizations
        .into_iter()
        .map(OrganizationResponse::from)
        .collect();
    Ok(Json(response))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrganizationResponse>> {
    let organization = state
        .organization_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("organization `{id}` not found")))?;

    Ok(Json(organization.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> AppResult<Json<OrganizationResponse>> {
    let organization = state
        .organization_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("organization `{id}` not found")))?;

    Ok(Json(organization.into()))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let removed = state.organization_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "organization `{id}` not found"
        )))
    }
}
