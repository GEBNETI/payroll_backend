use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct Job {
    pub id: Uuid,
    pub job_title: String,
    pub salary: f64,
    pub payroll_id: Uuid,
}

impl Job {
    pub fn new(id: Uuid, job_title: impl Into<String>, salary: f64, payroll_id: Uuid) -> Self {
        Self {
            id,
            job_title: job_title.into(),
            salary,
            payroll_id,
        }
    }
}
