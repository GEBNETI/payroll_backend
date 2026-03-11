use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};
use surrealdb::{engine::any::Any, types::{RecordId, SurrealValue}, Connection, Surreal};
use uuid::Uuid;

use crate::{
    domain::employee_payroll_concept::EmployeePayrollConcept,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::employee_payroll_concept::EmployeePayrollConceptRepository,
};

const EMPLOYEE_PAYROLL_CONCEPT_TABLE: &str = "employee_payroll_concept";

#[derive(Clone)]
pub struct SurrealEmployeePayrollConceptRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealEmployeePayrollConceptRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> EmployeePayrollConceptRepository for SurrealEmployeePayrollConceptRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(
        &self,
        id: Uuid,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
        amount: f64,
    ) -> AppResult<EmployeePayrollConcept> {
        let record: Option<Record> = self
            .client
            .create((EMPLOYEE_PAYROLL_CONCEPT_TABLE, id.to_string()))
            .content(build_record(employee_id, payroll_concept_id, amount))
            .await?;

        record_to_domain(record.ok_or_else(|| {
            AppError::internal("database did not return created employee payroll concept")
        })?)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<EmployeePayrollConcept>> {
        let record: Option<Record> = self
            .client
            .select((EMPLOYEE_PAYROLL_CONCEPT_TABLE, id.to_string()))
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_employee(&self, employee_id: Uuid) -> AppResult<Vec<EmployeePayrollConcept>> {
        let records: Vec<Record> = self.client.select(EMPLOYEE_PAYROLL_CONCEPT_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.employee_id == employee_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn fetch_by_employee_and_concept(
        &self,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        let records: Vec<Record> = self.client.select(EMPLOYEE_PAYROLL_CONCEPT_TABLE).await?;
        records
            .into_iter()
            .find(|record| {
                record.employee_id == employee_id.to_string()
                    && record.payroll_concept_id == payroll_concept_id.to_string()
            })
            .map(record_to_domain)
            .transpose()
    }

    async fn update_amount(
        &self,
        id: Uuid,
        amount: f64,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        let mut payload = Map::new();
        payload.insert("amount".to_string(), JsonValue::from(amount));

        let record: Option<Record> = self
            .client
            .update((EMPLOYEE_PAYROLL_CONCEPT_TABLE, id.to_string()))
            .merge(JsonValue::Object(payload))
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<Record> = self
            .client
            .delete((EMPLOYEE_PAYROLL_CONCEPT_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct Record {
    id: RecordId,
    employee_id: String,
    payroll_concept_id: String,
    amount: f64,
}

fn build_record(employee_id: Uuid, payroll_concept_id: Uuid, amount: f64) -> JsonValue {
    let mut object = Map::new();
    object.insert(
        "employee_id".to_string(),
        JsonValue::String(employee_id.to_string()),
    );
    object.insert(
        "payroll_concept_id".to_string(),
        JsonValue::String(payroll_concept_id.to_string()),
    );
    object.insert("amount".to_string(), JsonValue::from(amount));
    JsonValue::Object(object)
}

fn record_to_domain(record: Record) -> AppResult<EmployeePayrollConcept> {
    let id = parse_thing_id(&record.id.key, "stored employee payroll concept id")?;
    let employee_id = parse_uuid_field(
        &record.employee_id,
        "stored employee payroll concept employee_id",
    )?;
    let payroll_concept_id = parse_uuid_field(
        &record.payroll_concept_id,
        "stored employee payroll concept payroll_concept_id",
    )?;

    Ok(EmployeePayrollConcept::new(
        id,
        employee_id,
        payroll_concept_id,
        record.amount,
    ))
}

pub type SurrealAnyEmployeePayrollConceptRepository = SurrealEmployeePayrollConceptRepository<Any>;
