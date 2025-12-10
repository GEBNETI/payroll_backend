use serde::Deserialize;
use serde_json::json;
use surrealdb::{engine::any::Any, sql::Thing, Connection, Surreal};
use uuid::Uuid;

use crate::{
    domain::{
        payroll_concept::{PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType},
        payroll_history_detail::{NewPayrollHistoryDetailData, PayrollHistoryDetail},
    },
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::payroll_history_detail::PayrollHistoryDetailRepository,
};

const PAYROLL_HISTORY_DETAIL_TABLE: &str = "payroll_history_detail";

#[derive(Clone)]
pub struct SurrealPayrollHistoryDetailRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealPayrollHistoryDetailRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> PayrollHistoryDetailRepository for SurrealPayrollHistoryDetailRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, data: NewPayrollHistoryDetailData) -> AppResult<PayrollHistoryDetail> {
        let record: Option<PayrollHistoryDetailRecord> = self
            .client
            .create((PAYROLL_HISTORY_DETAIL_TABLE, data.id.to_string()))
            .content(json!({
                "payroll_history_id": data.payroll_history_id,
                "division_id": data.division_id,
                "division_name": data.division_name,
                "division_budget_code": data.division_budget_code,
                "job_id": data.job_id,
                "job_title": data.job_title,
                "job_salary": data.job_salary,
                "employee_id": data.employee_id,
                "employee_id_number": data.employee_id_number,
                "employee_last_name": data.employee_last_name,
                "employee_first_name": data.employee_first_name,
                "employee_hire_date": data.employee_hire_date.to_string(),
                "employee_salary": data.employee_salary,
                "employee_clasification": data.employee_clasification,
                "employee_hours": data.employee_hours,
                "employee_bank_account": data.employee_bank_account,
                "bank_id": data.bank_id,
                "bank_name": data.bank_name,
                "payroll_concept_id": data.payroll_concept_id,
                "payroll_concept_code": data.payroll_concept_code,
                "payroll_concept_name": data.payroll_concept_name,
                "payroll_concept_type": concept_type_to_string(&data.payroll_concept_type),
                "payroll_concept_scope": concept_scope_to_string(&data.payroll_concept_scope),
                "payroll_concept_period": concept_period_to_string(&data.payroll_concept_period),
                "amount": data.amount,
            }))
            .await?;

        record.map(record_to_domain).transpose()?.ok_or_else(|| {
            AppError::internal("database did not return created payroll history detail")
        })
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollHistoryDetail>> {
        let record: Option<PayrollHistoryDetailRecord> = self
            .client
            .select((PAYROLL_HISTORY_DETAIL_TABLE, id.to_string()))
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_payroll_history(
        &self,
        payroll_history_id: Uuid,
    ) -> AppResult<Vec<PayrollHistoryDetail>> {
        let mut result = self
            .client
            .query(
                "SELECT * FROM type::table($table) WHERE payroll_history_id = $payroll_history_id",
            )
            .bind(("table", PAYROLL_HISTORY_DETAIL_TABLE))
            .bind(("payroll_history_id", payroll_history_id.to_string()))
            .await?;
        let records: Vec<PayrollHistoryDetailRecord> = result.take(0)?;
        records.into_iter().map(record_to_domain).collect()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<PayrollHistoryDetailRecord> = self
            .client
            .delete((PAYROLL_HISTORY_DETAIL_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct PayrollHistoryDetailRecord {
    id: Thing,
    payroll_history_id: String,
    division_id: String,
    division_name: String,
    division_budget_code: String,
    job_id: String,
    job_title: String,
    job_salary: f64,
    employee_id: String,
    employee_id_number: String,
    employee_last_name: String,
    employee_first_name: String,
    employee_hire_date: String,
    employee_salary: f64,
    employee_clasification: String,
    employee_hours: i32,
    employee_bank_account: String,
    bank_id: String,
    bank_name: String,
    payroll_concept_id: String,
    payroll_concept_code: String,
    payroll_concept_name: String,
    payroll_concept_type: String,
    payroll_concept_scope: String,
    payroll_concept_period: String,
    amount: f64,
}

fn record_to_domain(record: PayrollHistoryDetailRecord) -> AppResult<PayrollHistoryDetail> {
    let id = parse_thing_id(&record.id.id, "stored payroll history detail id")?;
    let payroll_history_id = parse_uuid_field(
        &record.payroll_history_id,
        "stored payroll history detail payroll_history_id",
    )?;
    let division_id = parse_uuid_field(
        &record.division_id,
        "stored payroll history detail division_id",
    )?;
    let job_id = parse_uuid_field(&record.job_id, "stored payroll history detail job_id")?;
    let employee_id = parse_uuid_field(
        &record.employee_id,
        "stored payroll history detail employee_id",
    )?;
    let bank_id = parse_uuid_field(&record.bank_id, "stored payroll history detail bank_id")?;
    let payroll_concept_id = parse_uuid_field(
        &record.payroll_concept_id,
        "stored payroll history detail payroll_concept_id",
    )?;
    let employee_hire_date = parse_date(&record.employee_hire_date, "employee_hire_date")?;
    let payroll_concept_type = parse_concept_type(&record.payroll_concept_type)?;
    let payroll_concept_scope = parse_concept_scope(&record.payroll_concept_scope)?;
    let payroll_concept_period = parse_concept_period(&record.payroll_concept_period)?;

    Ok(PayrollHistoryDetail::new(NewPayrollHistoryDetailData {
        id,
        payroll_history_id,
        division_id,
        division_name: record.division_name,
        division_budget_code: record.division_budget_code,
        job_id,
        job_title: record.job_title,
        job_salary: record.job_salary,
        employee_id,
        employee_id_number: record.employee_id_number,
        employee_last_name: record.employee_last_name,
        employee_first_name: record.employee_first_name,
        employee_hire_date,
        employee_salary: record.employee_salary,
        employee_clasification: record.employee_clasification,
        employee_hours: record.employee_hours,
        employee_bank_account: record.employee_bank_account,
        bank_id,
        bank_name: record.bank_name,
        payroll_concept_id,
        payroll_concept_code: record.payroll_concept_code,
        payroll_concept_name: record.payroll_concept_name,
        payroll_concept_type,
        payroll_concept_scope,
        payroll_concept_period,
        amount: record.amount,
    }))
}

fn parse_date(value: &str, field: &str) -> AppResult<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::internal(format!("stored {field} is not a valid date")))
}

fn concept_type_to_string(t: &PayrollConceptType) -> &'static str {
    match t {
        PayrollConceptType::Earning => "earning",
        PayrollConceptType::Deduction => "deduction",
    }
}

fn concept_scope_to_string(s: &PayrollConceptScope) -> &'static str {
    match s {
        PayrollConceptScope::Global => "global",
        PayrollConceptScope::Individual => "individual",
    }
}

fn concept_period_to_string(p: &PayrollConceptPeriod) -> &'static str {
    match p {
        PayrollConceptPeriod::One => "1",
        PayrollConceptPeriod::Two => "2",
        PayrollConceptPeriod::Both => "both",
        PayrollConceptPeriod::Special => "special",
    }
}

fn parse_concept_type(value: &str) -> AppResult<PayrollConceptType> {
    match value {
        "earning" => Ok(PayrollConceptType::Earning),
        "deduction" => Ok(PayrollConceptType::Deduction),
        _ => Err(AppError::internal(format!(
            "unknown payroll concept type: {value}"
        ))),
    }
}

fn parse_concept_scope(value: &str) -> AppResult<PayrollConceptScope> {
    match value {
        "global" => Ok(PayrollConceptScope::Global),
        "individual" => Ok(PayrollConceptScope::Individual),
        _ => Err(AppError::internal(format!(
            "unknown payroll concept scope: {value}"
        ))),
    }
}

fn parse_concept_period(value: &str) -> AppResult<PayrollConceptPeriod> {
    match value {
        "1" => Ok(PayrollConceptPeriod::One),
        "2" => Ok(PayrollConceptPeriod::Two),
        "both" => Ok(PayrollConceptPeriod::Both),
        "special" => Ok(PayrollConceptPeriod::Special),
        _ => Err(AppError::internal(format!(
            "unknown payroll concept period: {value}"
        ))),
    }
}

pub type SurrealAnyPayrollHistoryDetailRepository = SurrealPayrollHistoryDetailRepository<Any>;
