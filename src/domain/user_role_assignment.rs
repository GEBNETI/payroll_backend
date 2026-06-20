use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct UserRoleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub organization_id: Option<Uuid>,
    pub payroll_id: Option<Uuid>,
    pub payroll_name: Option<String>,
}

impl UserRoleAssignment {
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        role_name: String,
        organization_id: Option<Uuid>,
        payroll_id: Option<Uuid>,
        payroll_name: Option<String>,
    ) -> Self {
        Self {
            id,
            user_id,
            role_id,
            role_name,
            organization_id,
            payroll_id,
            payroll_name,
        }
    }
}
