use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayrollHistoryStatus {
    Draft,
    Finalized,
    Archived,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct PayrollHistory {
    pub id: Uuid,
    pub title: String,
    pub period: String,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub end_date: NaiveDate,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub payroll_id: Uuid,
    pub payroll_name: String,
    pub status: PayrollHistoryStatus,
    pub total_employees: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
}

#[derive(Clone, Debug)]
pub struct NewPayrollHistoryData {
    pub id: Uuid,
    pub title: String,
    pub period: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub payroll_id: Uuid,
    pub payroll_name: String,
    pub status: PayrollHistoryStatus,
    pub total_employees: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
}

impl PayrollHistory {
    pub fn new(data: NewPayrollHistoryData) -> Self {
        Self {
            id: data.id,
            title: data.title,
            period: data.period,
            start_date: data.start_date,
            end_date: data.end_date,
            created_at: data.created_at,
            organization_id: data.organization_id,
            organization_name: data.organization_name,
            payroll_id: data.payroll_id,
            payroll_name: data.payroll_name,
            status: data.status,
            total_employees: data.total_employees,
            total_earnings: data.total_earnings,
            total_deductions: data.total_deductions,
        }
    }
}
