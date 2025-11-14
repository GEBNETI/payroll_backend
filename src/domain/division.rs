use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct Division {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
}

impl Division {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        budget_code: impl Into<String>,
        payroll_id: Uuid,
        parent_division_id: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            budget_code: budget_code.into(),
            payroll_id,
            parent_division_id,
        }
    }
}
