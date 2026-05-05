use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::employee_payroll_concept::EmployeePayrollConcept,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::employee_payroll_concept::EmployeePayrollConceptRepository,
};

const TABLE: &str = "employee_payroll_concept";

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
        let _: Option<JsonValue> = self
            .client
            .create((TABLE, id.to_string()))
            .content(build_record(employee_id, payroll_concept_id, amount))
            .await?;

        self.fetch(id).await?.ok_or_else(|| {
            AppError::internal("database did not return created employee payroll concept")
        })
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<EmployeePayrollConcept>> {
        let rid = RecordId::new(TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM employee_payroll_concept WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<Record>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_employee(&self, employee_id: Uuid) -> AppResult<Vec<EmployeePayrollConcept>> {
        let mut response = self
            .client
            .query("SELECT * FROM employee_payroll_concept WHERE employee_id = $employee_id")
            .bind(("employee_id", employee_id.to_string()))
            .await?;

        let result: Result<Vec<Record>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_employee_and_concept(
        &self,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        let mut response = self
            .client
            .query(
                "SELECT * FROM employee_payroll_concept \
                 WHERE employee_id = $employee_id AND payroll_concept_id = $concept_id",
            )
            .bind(("employee_id", employee_id.to_string()))
            .bind(("concept_id", payroll_concept_id.to_string()))
            .await?;

        let result: Result<Option<Record>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update_amount(
        &self,
        id: Uuid,
        amount: f64,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        let mut payload = Map::new();
        payload.insert("amount".to_string(), JsonValue::from(amount));

        let _: Option<JsonValue> = self
            .client
            .update((TABLE, id.to_string()))
            .merge(JsonValue::Object(payload))
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self.client.delete((TABLE, id.to_string())).await?;

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
