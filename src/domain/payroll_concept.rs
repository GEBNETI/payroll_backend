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

/// Data needed to create a PayrollConcept.
#[derive(Clone, Debug)]
pub struct NewPayrollConceptData {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
    pub period: PayrollConceptPeriod,
    pub active: bool,
    pub payroll_id: Uuid,
}

impl PayrollConcept {
    pub fn new(data: NewPayrollConceptData) -> Self {
        Self {
            id: data.id,
            code: data.code,
            name: data.name,
            concept_type: data.concept_type,
            scope: data.scope,
            period: data.period,
            active: data.active,
            payroll_id: data.payroll_id,
        }
    }
}
