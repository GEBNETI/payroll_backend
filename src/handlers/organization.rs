use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
    extractors::auth::AuthUser,
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

impl From<CreateOrganizationRequest> for CreateOrganizationParams {
    fn from(req: CreateOrganizationRequest) -> Self {
        Self { name: req.name }
    }
}

impl From<UpdateOrganizationRequest> for UpdateOrganizationParams {
    fn from(req: UpdateOrganizationRequest) -> Self {
        Self { name: req.name }
    }
}

#[utoipa::path(
    post,
    path = "/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created", body = OrganizationResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser access required"),
    ),
    tag = "Organizations",
    operation_id = "create_organization"
)]
pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateOrganizationRequest>,
) -> AppResult<(StatusCode, Json<OrganizationResponse>)> {
    auth_user.require_superuser()?;
    let organization = state.organization_service().create(payload.into()).await?;
    Ok((StatusCode::CREATED, Json(organization.into())))
}

#[utoipa::path(
    get,
    path = "/organizations",
    responses(
        (status = 200, description = "List organizations", body = [OrganizationResponse]),
        (status = 401, description = "Not authenticated"),
    ),
    tag = "Organizations",
    operation_id = "list_organizations"
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<Vec<OrganizationResponse>>> {
    let organizations = state.organization_service().list().await?;

    if auth_user.is_superuser {
        return Ok(Json(organizations.into_iter().map(OrganizationResponse::from).collect()));
    }

    // Non-superusers see only organizations they have an assignment in
    let accessible_org_ids: std::collections::HashSet<Uuid> = auth_user
        .assignments
        .iter()
        .filter_map(|a| a.organization_id)
        .collect();

    let filtered = organizations
        .into_iter()
        .filter(|o| accessible_org_ids.contains(&o.id))
        .map(OrganizationResponse::from)
        .collect();

    Ok(Json(filtered))
}

#[utoipa::path(
    get,
    path = "/organizations/{id}",
    params(OrganizationPathParams),
    responses(
        (status = 200, description = "Get organization", body = OrganizationResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Organization not found"),
    ),
    tag = "Organizations",
    operation_id = "get_organization"
)]
pub async fn get(
    State(state): State<AppState>,
    _auth_user: AuthUser,
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
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser access required"),
        (status = 404, description = "Organization not found"),
    ),
    tag = "Organizations",
    operation_id = "update_organization"
)]
pub async fn update(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<OrganizationPathParams>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> AppResult<Json<OrganizationResponse>> {
    auth_user.require_superuser()?;
    let id = params.id;
    let organization = state
        .organization_service()
        .update(id, payload.into())
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
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser access required"),
        (status = 404, description = "Organization not found"),
    ),
    tag = "Organizations",
    operation_id = "delete_organization"
)]
pub async fn delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<OrganizationPathParams>,
) -> AppResult<StatusCode> {
    auth_user.require_superuser()?;
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
