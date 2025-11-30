use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct EmployeePayrollConcept {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

impl EmployeePayrollConcept {
    pub fn new(id: Uuid, employee_id: Uuid, payroll_concept_id: Uuid, amount: f64) -> Self {
        Self {
            id,
            employee_id,
            payroll_concept_id,
            amount,
        }
    }
}
