use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{domain::user_role_assignment::UserRoleAssignment, error::AppResult};

#[derive(Debug, Clone)]
pub struct AssignRoleParams {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub organization_id: Option<Uuid>,
    pub payroll_id: Option<Uuid>,
    pub payroll_name: Option<String>,
}

#[async_trait]
pub trait UserRoleAssignmentRepository: Send + Sync {
    async fn insert(&self, id: Uuid, params: AssignRoleParams) -> AppResult<UserRoleAssignment>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<UserRoleAssignment>>;
    async fn fetch_for_user(&self, user_id: Uuid) -> AppResult<Vec<UserRoleAssignment>>;
    async fn fetch_for_org(&self, org_id: Uuid) -> AppResult<Vec<UserRoleAssignment>>;
    async fn fetch_for_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<UserRoleAssignment>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct UserRoleAssignmentService {
    repository: Arc<dyn UserRoleAssignmentRepository>,
}

impl UserRoleAssignmentService {
    pub fn new(repository: Arc<dyn UserRoleAssignmentRepository>) -> Self {
        Self { repository }
    }

    pub async fn assign(&self, params: AssignRoleParams) -> AppResult<UserRoleAssignment> {
        let id = Uuid::new_v4();
        self.repository.insert(id, params).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<UserRoleAssignment>> {
        self.repository.fetch(id).await
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        self.repository.fetch_for_user(user_id).await
    }

    pub async fn list_for_org(&self, org_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        self.repository.fetch_for_org(org_id).await
    }

    pub async fn list_for_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        self.repository.fetch_for_payroll(payroll_id).await
    }

    pub async fn revoke(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
