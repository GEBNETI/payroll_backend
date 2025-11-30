use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayrollConceptType {
    Earning,
    Deduction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayrollConceptScope {
    Global,
    Individual,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub enum PayrollConceptPeriod {
    #[serde(rename = "1")]
    #[schema(rename = "1")]
    One,
    #[serde(rename = "2")]
    #[schema(rename = "2")]
    Two,
    #[serde(rename = "both")]
    Both,
    #[serde(rename = "special")]
    Special,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct PayrollConcept {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
    pub period: PayrollConceptPeriod,
    pub active: bool,
    pub payroll_id: Uuid,
}

impl PayrollConcept {
    pub fn new(
        id: Uuid,
        code: impl Into<String>,
        name: impl Into<String>,
        concept_type: PayrollConceptType,
        scope: PayrollConceptScope,
        period: PayrollConceptPeriod,
        active: bool,
        payroll_id: Uuid,
    ) -> Self {
        Self {
            id,
            code: code.into(),
            name: name.into(),
            concept_type,
            scope,
            period,
            active,
            payroll_id,
        }
    }
}
