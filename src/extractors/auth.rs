use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};
use uuid::Uuid;

use crate::{
    domain::{
        role::{ORGANIZATION_MANAGER_ROLE, PAYROLL_REPORT_ROLE, PAYROLL_USER_ROLE},
        user_role_assignment::UserRoleAssignment,
    },
    error::{AppError, AppResult},
    server::AppState,
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
    pub name: String,
    pub is_superuser: bool,
    pub assignments: Vec<UserRoleAssignment>,
}

impl AuthUser {
    pub fn is_org_manager(&self) -> bool {
        self.is_superuser
            || self
                .assignments
                .iter()
                .any(|a| a.role_name == ORGANIZATION_MANAGER_ROLE)
    }

    pub fn managed_org_ids(&self) -> Vec<Uuid> {
        self.assignments
            .iter()
            .filter(|a| a.role_name == ORGANIZATION_MANAGER_ROLE)
            .filter_map(|a| a.organization_id)
            .collect()
    }

    pub fn require_superuser(&self) -> AppResult<()> {
        if self.is_superuser {
            Ok(())
        } else {
            Err(AppError::forbidden("superuser access required"))
        }
    }

    pub fn require_org_write(&self, org_id: Uuid) -> AppResult<()> {
        if self.is_superuser {
            return Ok(());
        }
        let ok = self
            .assignments
            .iter()
            .any(|a| a.organization_id == Some(org_id) && a.role_name == ORGANIZATION_MANAGER_ROLE);
        if ok {
            Ok(())
        } else {
            Err(AppError::forbidden(
                "insufficient permissions for this organization",
            ))
        }
    }

    pub fn require_org_read(&self, org_id: Uuid) -> AppResult<()> {
        if self.is_superuser {
            return Ok(());
        }
        let ok = self
            .assignments
            .iter()
            .any(|a| a.organization_id == Some(org_id));
        if ok {
            Ok(())
        } else {
            Err(AppError::forbidden(
                "access to this organization is not permitted",
            ))
        }
    }

    pub fn require_payroll_write(&self, payroll_id: Uuid) -> AppResult<()> {
        if self.is_superuser {
            return Ok(());
        }
        let ok = self.assignments.iter().any(|a| {
            a.payroll_id == Some(payroll_id)
                && (a.role_name == PAYROLL_USER_ROLE || a.role_name == ORGANIZATION_MANAGER_ROLE)
        });
        if ok {
            Ok(())
        } else {
            Err(AppError::forbidden(
                "write access to this payroll is not permitted",
            ))
        }
    }

    pub fn require_payroll_read(&self, payroll_id: Uuid) -> AppResult<()> {
        if self.is_superuser {
            return Ok(());
        }
        let ok = self.assignments.iter().any(|a| {
            a.payroll_id == Some(payroll_id)
                && (a.role_name == PAYROLL_USER_ROLE
                    || a.role_name == PAYROLL_REPORT_ROLE
                    || a.role_name == ORGANIZATION_MANAGER_ROLE)
        });
        if ok {
            Ok(())
        } else {
            Err(AppError::forbidden(
                "read access to this payroll is not permitted",
            ))
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(&parts.headers)?;
        let claims = state.auth_service().validate_access_token(token)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::unauthorized("invalid token subject"))?;

        let assignments = state
            .user_role_assignment_service()
            .list_for_user(user_id)
            .await?;

        Ok(AuthUser {
            user_id,
            username: claims.username,
            name: claims.name,
            is_superuser: claims.is_superuser,
            assignments,
        })
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> AppResult<&str> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::unauthorized("missing Authorization header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("invalid Authorization header encoding"))?;

    auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("Authorization header must use Bearer scheme"))
}
