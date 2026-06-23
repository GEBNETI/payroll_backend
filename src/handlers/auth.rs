use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    Json,
};
use crate::handlers::user::UserRoleAssignmentResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{role::SUPERUSER_ROLE, user::User},
    error::{AppError, AppResult},
    extractors::auth::AuthUser,
    server::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: UserPayload,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPayload {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub name: String,
    pub is_active: bool,
    pub is_superuser: bool,
}

impl UserPayload {
    pub fn from_user(user: User, is_superuser: bool) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            name: user.name,
            is_active: user.is_active,
            is_superuser,
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "Auth",
    operation_id = "auth_login"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<LoginResponse>)> {
    let user = state
        .auth_service()
        .authenticate(&payload.username, &payload.password)
        .await?;

    let assignments = state
        .user_role_assignment_service()
        .list_for_user(user.id)
        .await?;
    let is_superuser = assignments.iter().any(|a| a.role_name == SUPERUSER_ROLE);

    let access_token = state
        .auth_service()
        .generate_access_token(&user, is_superuser)?;
    let refresh_token = state.auth_service().generate_refresh_token(&user)?;

    let expiry = state.auth_service().refresh_expiry_seconds();
    let cookie = format!(
        "refresh_token={refresh_token}; HttpOnly; Path=/auth/refresh; Max-Age={expiry}; SameSite=Lax"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        cookie
            .parse()
            .map_err(|_| AppError::internal("failed to set cookie"))?,
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(LoginResponse {
            access_token,
            user: UserPayload::from_user(user, is_superuser),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed", body = RefreshResponse),
        (status = 401, description = "Invalid or missing refresh token"),
    ),
    tag = "Auth",
    operation_id = "auth_refresh"
)]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<RefreshResponse>> {
    let refresh_token = extract_refresh_token(&headers)?;
    let claims = state.auth_service().validate_refresh_token(refresh_token)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("invalid token subject"))?;

    let user = state
        .user_service()
        .get(user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("user not found"))?;

    if !user.is_active {
        return Err(AppError::unauthorized("account is disabled"));
    }

    let assignments = state
        .user_role_assignment_service()
        .list_for_user(user_id)
        .await?;
    let is_superuser = assignments.iter().any(|a| a.role_name == SUPERUSER_ROLE);

    let access_token = state
        .auth_service()
        .generate_access_token(&user, is_superuser)?;

    Ok(Json(RefreshResponse { access_token }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 204, description = "Logged out"),
    ),
    tag = "Auth",
    operation_id = "auth_logout"
)]
pub async fn logout() -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    let clear_cookie =
        "refresh_token=; HttpOnly; Path=/auth/refresh; Max-Age=0; SameSite=Strict";
    if let Ok(v) = clear_cookie.parse() {
        headers.insert(header::SET_COOKIE, v);
    }
    (StatusCode::NO_CONTENT, headers)
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user", body = UserPayload),
        (status = 401, description = "Not authenticated"),
    ),
    tag = "Auth",
    operation_id = "auth_me"
)]
pub async fn me(auth_user: AuthUser) -> AppResult<Json<UserPayload>> {
    Ok(Json(UserPayload {
        id: auth_user.user_id,
        username: auth_user.username,
        name: auth_user.name,
        email: String::new(),
        is_active: true,
        is_superuser: auth_user.is_superuser,
    }))
}

#[utoipa::path(
    get,
    path = "/auth/assignments",
    responses(
        (status = 200, description = "Current user's role assignments", body = [UserRoleAssignmentResponse]),
        (status = 401, description = "Not authenticated"),
    ),
    tag = "Auth",
    operation_id = "auth_assignments"
)]
pub async fn my_assignments(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<Vec<UserRoleAssignmentResponse>>> {
    let assignments = state
        .user_role_assignment_service()
        .list_for_user(auth_user.user_id)
        .await?;
    Ok(Json(
        assignments
            .into_iter()
            .map(UserRoleAssignmentResponse::from)
            .collect(),
    ))
}

fn extract_refresh_token(headers: &HeaderMap) -> AppResult<&str> {
    let cookie_header = headers
        .get(header::COOKIE)
        .ok_or_else(|| AppError::unauthorized("missing refresh token cookie"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("invalid cookie header"))?;

    cookie_header
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("refresh_token=")
        })
        .ok_or_else(|| AppError::unauthorized("refresh_token cookie not found"))
}
