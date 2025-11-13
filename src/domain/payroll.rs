use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Payroll {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
}

impl Payroll {
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        organization_id: Uuid,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            organization_id,
        }
    }
}
