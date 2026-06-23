use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::user_role_assignment::UserRoleAssignment,
    error::{AppError, AppResult},
    extractors::auth::AuthUser,
    server::AppState,
    services::{
        auth::AuthService,
        user::{InsertUserParams, UpdateUserParams},
        user_role_assignment::AssignRoleParams,
    },
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub role_name: String,
    pub organization_id: Option<Uuid>,
    pub payroll_id: Option<Uuid>,
    pub payroll_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserRoleAssignmentResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub organization_id: Option<Uuid>,
    pub payroll_id: Option<Uuid>,
    pub payroll_name: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct UserPathParams {
    pub id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AssignmentPathParams {
    pub id: Uuid,
    pub assignment_id: Uuid,
}

impl From<crate::domain::user::User> for UserResponse {
    fn from(u: crate::domain::user::User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            name: u.name,
            is_active: u.is_active,
        }
    }
}

impl From<UserRoleAssignment> for UserRoleAssignmentResponse {
    fn from(a: UserRoleAssignment) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            role_id: a.role_id,
            role_name: a.role_name,
            organization_id: a.organization_id,
            payroll_id: a.payroll_id,
            payroll_name: a.payroll_name,
        }
    }
}

/// Collects all user IDs visible to the given org manager across all their managed orgs.
/// Includes users with:
/// - an org-scoped assignment (organization_id in managed orgs), OR
/// - a payroll-scoped assignment for any payroll belonging to a managed org
async fn visible_user_ids_for_org_manager(
    managed: &[Uuid],
    state: &AppState,
) -> AppResult<HashSet<Uuid>> {
    let mut ids: HashSet<Uuid> = HashSet::new();

    for &org_id in managed {
        // Org-scoped assignments (OrganizationManager stored with organization_id)
        for a in state.user_role_assignment_service().list_for_org(org_id).await? {
            ids.insert(a.user_id);
        }
        // Payroll-scoped assignments (PayrollUser/PayrollReport stored with payroll_id)
        let payrolls = state.payroll_service().list(org_id).await?;
        for payroll in payrolls {
            for a in state.user_role_assignment_service().list_for_payroll(payroll.id).await? {
                ids.insert(a.user_id);
            }
        }
    }

    Ok(ids)
}

/// Returns true if the requesting org manager can access the given user.
/// An org manager can access a user if:
/// - the user is visible via their org or payroll assignments, OR
/// - the user has no assignments at all (newly created, awaiting assignment)
async fn org_manager_can_access_user(
    auth_user: &AuthUser,
    user_id: Uuid,
    state: &AppState,
) -> AppResult<bool> {
    let managed = auth_user.managed_org_ids();
    if managed.is_empty() {
        return Ok(false);
    }
    let user_assignments = state
        .user_role_assignment_service()
        .list_for_user(user_id)
        .await?;
    if user_assignments.is_empty() {
        return Ok(true);
    }
    let visible = visible_user_ids_for_org_manager(&managed, state).await?;
    Ok(visible.contains(&user_id))
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
    ),
    tag = "Users",
    operation_id = "create_user"
)]
pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    if !auth_user.is_org_manager() {
        return Err(AppError::forbidden("superuser or organization manager access required"));
    }

    let password_hash = AuthService::hash_password(&payload.password)?;
    let user = state
        .user_service()
        .create(InsertUserParams {
            username: payload.username,
            email: payload.email,
            password_hash,
            name: payload.name,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List users", body = [UserResponse]),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
    ),
    tag = "Users",
    operation_id = "list_users"
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<Vec<UserResponse>>> {
    if auth_user.is_superuser {
        let users = state.user_service().list().await?;
        return Ok(Json(users.into_iter().map(UserResponse::from).collect()));
    }

    let managed = auth_user.managed_org_ids();
    if managed.is_empty() {
        return Err(AppError::forbidden("superuser or organization manager access required"));
    }

    let visible_user_ids = visible_user_ids_for_org_manager(&managed, &state).await?;

    let all_users = state.user_service().list().await?;
    let filtered: Vec<_> = all_users
        .into_iter()
        .filter(|u| visible_user_ids.contains(&u.id))
        .collect();

    Ok(Json(filtered.into_iter().map(UserResponse::from).collect()))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(UserPathParams),
    responses(
        (status = 200, description = "Get user", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users",
    operation_id = "get_user"
)]
pub async fn get(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<UserPathParams>,
) -> AppResult<Json<UserResponse>> {
    if !auth_user.is_superuser && !org_manager_can_access_user(&auth_user, params.id, &state).await? {
        return Err(AppError::forbidden("access to this user is not permitted"));
    }

    let user = state
        .user_service()
        .get(params.id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("user `{}` not found", params.id)))?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(UserPathParams),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users",
    operation_id = "update_user"
)]
pub async fn update(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<UserPathParams>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    if !auth_user.is_superuser && !org_manager_can_access_user(&auth_user, params.id, &state).await? {
        return Err(AppError::forbidden("access to this user is not permitted"));
    }

    let password_hash = payload
        .password
        .as_deref()
        .map(AuthService::hash_password)
        .transpose()?;

    let user = state
        .user_service()
        .update(
            params.id,
            UpdateUserParams {
                email: payload.email,
                name: payload.name,
                password_hash,
                is_active: payload.is_active,
            },
        )
        .await?
        .ok_or_else(|| AppError::not_found(format!("user `{}` not found", params.id)))?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(UserPathParams),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser access required"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users",
    operation_id = "delete_user"
)]
pub async fn delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<UserPathParams>,
) -> AppResult<StatusCode> {
    auth_user.require_superuser()?;

    let removed = state.user_service().delete(params.id).await?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "user `{}` not found",
            params.id
        )))
    }
}

#[utoipa::path(
    get,
    path = "/users/{id}/assignments",
    params(UserPathParams),
    responses(
        (status = 200, description = "List role assignments for user", body = [UserRoleAssignmentResponse]),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
    ),
    tag = "Users",
    operation_id = "list_user_assignments"
)]
pub async fn list_assignments(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<UserPathParams>,
) -> AppResult<Json<Vec<UserRoleAssignmentResponse>>> {
    if !auth_user.is_superuser && !org_manager_can_access_user(&auth_user, params.id, &state).await? {
        return Err(AppError::forbidden("access to this user is not permitted"));
    }

    let assignments = state
        .user_role_assignment_service()
        .list_for_user(params.id)
        .await?;

    Ok(Json(
        assignments
            .into_iter()
            .map(UserRoleAssignmentResponse::from)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/users/{id}/assignments",
    params(UserPathParams),
    request_body = AssignRoleRequest,
    responses(
        (status = 201, description = "Role assigned", body = UserRoleAssignmentResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
        (status = 404, description = "Role not found"),
    ),
    tag = "Users",
    operation_id = "assign_user_role"
)]
pub async fn assign_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<UserPathParams>,
    Json(payload): Json<AssignRoleRequest>,
) -> AppResult<(StatusCode, Json<UserRoleAssignmentResponse>)> {
    if !auth_user.is_superuser {
        let managed = auth_user.managed_org_ids();
        if managed.is_empty() {
            return Err(AppError::forbidden("superuser or organization manager access required"));
        }
        // Org managers can only assign roles scoped to their own orgs
        match payload.organization_id {
            Some(org_id) if managed.contains(&org_id) => {}
            _ => return Err(AppError::forbidden("org managers can only assign roles within their managed organizations")),
        }
        if !org_manager_can_access_user(&auth_user, params.id, &state).await? {
            return Err(AppError::forbidden("access to this user is not permitted"));
        }
    }

    let role = state
        .role_service()
        .get_by_name(&payload.role_name)
        .await?
        .ok_or_else(|| AppError::not_found(format!("role `{}` not found", payload.role_name)))?;

    let assignment = state
        .user_role_assignment_service()
        .assign(AssignRoleParams {
            user_id: params.id,
            role_id: role.id,
            role_name: role.name,
            organization_id: payload.organization_id,
            payroll_id: payload.payroll_id,
            payroll_name: payload.payroll_name,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(assignment.into())))
}

#[utoipa::path(
    delete,
    path = "/users/{id}/assignments/{assignment_id}",
    params(AssignmentPathParams),
    responses(
        (status = 204, description = "Assignment removed"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Superuser or OrganizationManager access required"),
        (status = 404, description = "Assignment not found"),
    ),
    tag = "Users",
    operation_id = "revoke_user_role"
)]
pub async fn revoke_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(params): Path<AssignmentPathParams>,
) -> AppResult<StatusCode> {
    if !auth_user.is_superuser {
        let managed = auth_user.managed_org_ids();
        if managed.is_empty() {
            return Err(AppError::forbidden("superuser or organization manager access required"));
        }
        // Verify the assignment belongs to one of the manager's orgs
        let assignment = state
            .user_role_assignment_service()
            .get(params.assignment_id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("assignment `{}` not found", params.assignment_id)))?;
        match assignment.organization_id {
            Some(org_id) if managed.contains(&org_id) => {}
            _ => return Err(AppError::forbidden("org managers can only revoke assignments within their managed organizations")),
        }
    }

    let removed = state
        .user_role_assignment_service()
        .revoke(params.assignment_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "assignment `{}` not found",
            params.assignment_id
        )))
    }
}
