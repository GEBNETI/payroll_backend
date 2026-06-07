use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::payroll_concept::{
    PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct PayrollHistoryDetail {
    pub id: Uuid,
    pub payroll_history_id: Uuid,
    // Division info (denormalized)
    pub division_id: Uuid,
    pub division_name: String,
    pub division_budget_code: String,
    // Job info (denormalized)
    pub job_id: Uuid,
    pub job_title: String,
    pub job_salary: f64,
    // Employee info (denormalized)
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub employee_last_name: String,
    pub employee_first_name: String,
    #[schema(value_type = String, format = Date)]
    pub employee_hire_date: NaiveDate,
    pub employee_salary: f64,
    pub employee_classification: String,
    pub employee_hours: i32,
    pub employee_bank_account: String,
    // Bank info (denormalized)
    pub bank_id: Uuid,
    pub bank_name: String,
    // Payroll concept info (denormalized)
    pub payroll_concept_id: Uuid,
    pub payroll_concept_code: String,
    pub payroll_concept_name: String,
    pub payroll_concept_type: PayrollConceptType,
    pub payroll_concept_scope: PayrollConceptScope,
    pub payroll_concept_period: PayrollConceptPeriod,
    // Amount
    pub amount: f64,
}

#[derive(Clone, Debug)]
pub struct NewPayrollHistoryDetailData {
    pub id: Uuid,
    pub payroll_history_id: Uuid,
    pub division_id: Uuid,
    pub division_name: String,
    pub division_budget_code: String,
    pub job_id: Uuid,
    pub job_title: String,
    pub job_salary: f64,
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub employee_last_name: String,
    pub employee_first_name: String,
    pub employee_hire_date: NaiveDate,
    pub employee_salary: f64,
    pub employee_classification: String,
    pub employee_hours: i32,
    pub employee_bank_account: String,
    pub bank_id: Uuid,
    pub bank_name: String,
    pub payroll_concept_id: Uuid,
    pub payroll_concept_code: String,
    pub payroll_concept_name: String,
    pub payroll_concept_type: PayrollConceptType,
    pub payroll_concept_scope: PayrollConceptScope,
    pub payroll_concept_period: PayrollConceptPeriod,
    pub amount: f64,
}

impl PayrollHistoryDetail {
    pub fn new(data: NewPayrollHistoryDetailData) -> Self {
        Self {
            id: data.id,
            payroll_history_id: data.payroll_history_id,
            division_id: data.division_id,
            division_name: data.division_name,
            division_budget_code: data.division_budget_code,
            job_id: data.job_id,
            job_title: data.job_title,
            job_salary: data.job_salary,
            employee_id: data.employee_id,
            employee_id_number: data.employee_id_number,
            employee_last_name: data.employee_last_name,
            employee_first_name: data.employee_first_name,
            employee_hire_date: data.employee_hire_date,
            employee_salary: data.employee_salary,
            employee_classification: data.employee_classification,
            employee_hours: data.employee_hours,
            employee_bank_account: data.employee_bank_account,
            bank_id: data.bank_id,
            bank_name: data.bank_name,
            payroll_concept_id: data.payroll_concept_id,
            payroll_concept_code: data.payroll_concept_code,
            payroll_concept_name: data.payroll_concept_name,
            payroll_concept_type: data.payroll_concept_type,
            payroll_concept_scope: data.payroll_concept_scope,
            payroll_concept_period: data.payroll_concept_period,
            amount: data.amount,
        }
    }
}
