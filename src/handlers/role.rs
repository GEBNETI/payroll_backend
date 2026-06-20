use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::role::Role,
    error::{AppError, AppResult},
    extractors::auth::AuthUser,
    server::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub label: String,
}

#[derive(Debug, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct RolePathParams {
    pub id: Uuid,
}

impl From<Role> for RoleResponse {
    fn from(r: Role) -> Self {
        let label = role_label(&r.name);
        Self { id: r.id, name: r.name, label }
    }
}

fn role_label(name: &str) -> String {
    match name {
        "Superuser" => "Full access",
        "OrganizationManager" => "Organization access",
        "PayrollUser" => "Payroll access",
        "PayrollReport" => "Payroll readonly access",
        other => other,
    }
    .to_string()
}

#[utoipa::path(
    get,
    path = "/roles",
    responses(
        (status = 200, description = "List roles", body = [RoleResponse]),
        (status = 401, description = "Not authenticated"),
    ),
    tag = "Roles",
    operation_id = "list_roles"
)]
pub async fn list(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> AppResult<Json<Vec<RoleResponse>>> {
    let roles = state.role_service().list().await?;
    Ok(Json(roles.into_iter().map(RoleResponse::from).collect()))
}

#[utoipa::path(
    get,
    path = "/roles/{id}",
    params(RolePathParams),
    responses(
        (status = 200, description = "Get role", body = RoleResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Role not found"),
    ),
    tag = "Roles",
    operation_id = "get_role"
)]
pub async fn get(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(params): Path<RolePathParams>,
) -> AppResult<Json<RoleResponse>> {
    let role = state
        .role_service()
        .get(params.id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("role `{}` not found", params.id)))?;
    Ok(Json(role.into()))
}
