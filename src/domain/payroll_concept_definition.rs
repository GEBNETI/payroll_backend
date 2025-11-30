use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct PayrollConceptDefinition {
    pub id: Uuid,
    pub payroll_concept_id: Uuid,
    pub formula: String,
    pub condition: String,
}

impl PayrollConceptDefinition {
    pub fn new(
        id: Uuid,
        payroll_concept_id: Uuid,
        formula: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        Self {
            id,
            payroll_concept_id,
            formula: formula.into(),
            condition: condition.into(),
        }
    }
}
