use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub const SUPERUSER_ROLE: &str = "Superuser";
pub const ORGANIZATION_MANAGER_ROLE: &str = "OrganizationManager";
pub const PAYROLL_USER_ROLE: &str = "PayrollUser";
pub const PAYROLL_REPORT_ROLE: &str = "PayrollReport";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}

impl Role {
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}
