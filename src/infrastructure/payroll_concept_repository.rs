use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::payroll_concept::{
        NewPayrollConceptData, PayrollConcept, PayrollConceptPeriod, PayrollConceptScope,
        PayrollConceptType,
    },
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::payroll_concept::{
        InsertPayrollConceptParams, PayrollConceptRepository, UpdatePayrollConceptParams,
    },
};

const PAYROLL_CONCEPT_TABLE: &str = "payroll_concept";

const SELECT_PAYROLL_CONCEPT_FIELDS: &str =
    "SELECT id, code, name, concept_type, scope, period, active, payroll_id";

#[derive(Clone)]
pub struct SurrealPayrollConceptRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealPayrollConceptRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> PayrollConceptRepository for SurrealPayrollConceptRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, params: InsertPayrollConceptParams) -> AppResult<PayrollConcept> {
        // NOTE: field is stored as "concept_type", not "type" — "type" is a reserved
        // keyword in SurrealDB and is stripped from returned records.
        let _: Option<JsonValue> = self
            .client
            .create((PAYROLL_CONCEPT_TABLE, params.id.to_string()))
            .content(json!({
                "code": params.code,
                "name": params.name,
                "concept_type": params.concept_type,
                "scope": params.scope,
                "period": params.period,
                "active": params.active,
                "payroll_id": params.payroll_id,
            }))
            .await?;

        self.fetch(params.id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created payroll concept"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollConcept>> {
        let rid = RecordId::new(PAYROLL_CONCEPT_TABLE, id.to_string());
        let mut response = self
            .client
            .query(format!(
                "{SELECT_PAYROLL_CONCEPT_FIELDS} FROM payroll_concept WHERE id = $rid"
            ))
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<PayrollConceptRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<PayrollConcept>> {
        let mut response = self
            .client
            .query(format!(
                "{SELECT_PAYROLL_CONCEPT_FIELDS} FROM payroll_concept WHERE payroll_id = $payroll_id"
            ))
            .bind(("payroll_id", payroll_id.to_string()))
            .await?;

        let result: Result<Vec<PayrollConceptRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        params: UpdatePayrollConceptParams,
    ) -> AppResult<Option<PayrollConcept>> {
        let payload = build_update_payload(params)?;

        let _: Option<JsonValue> = self
            .client
            .update((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self
            .client
            .delete((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct PayrollConceptRecord {
    id: RecordId,
    code: String,
    name: String,
    concept_type: String,
    scope: String,
    period: String,
    active: bool,
    payroll_id: String,
}

fn record_to_domain(record: PayrollConceptRecord) -> AppResult<PayrollConcept> {
    let id = parse_thing_id(&record.id.key, "stored payroll concept id")?;
    let payroll_id = parse_uuid_field(&record.payroll_id, "stored payroll concept payroll_id")?;
    let concept_type = parse_concept_type(&record.concept_type)?;
    let scope = parse_concept_scope(&record.scope)?;
    let period = parse_concept_period(&record.period)?;

    Ok(PayrollConcept::new(NewPayrollConceptData {
        id,
        code: record.code,
        name: record.name,
        concept_type,
        scope,
        period,
        active: record.active,
        payroll_id,
    }))
}

fn build_update_payload(params: UpdatePayrollConceptParams) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(code) = params.code {
        object.insert("code".to_string(), JsonValue::String(code));
    }
    if let Some(name) = params.name {
        object.insert("name".to_string(), JsonValue::String(name));
    }
    if let Some(concept_type) = params.concept_type {
        let value = serde_json::to_value(concept_type).map_err(|_| {
            AppError::internal("failed to serialize concept type for update payload")
        })?;
        object.insert("concept_type".to_string(), value);
    }
    if let Some(scope) = params.scope {
        let value = serde_json::to_value(scope).map_err(|_| {
            AppError::internal("failed to serialize concept scope for update payload")
        })?;
        object.insert("scope".to_string(), value);
    }
    if let Some(period) = params.period {
        let value = serde_json::to_value(period).map_err(|_| {
            AppError::internal("failed to serialize concept period for update payload")
        })?;
        object.insert("period".to_string(), value);
    }
    if let Some(active) = params.active {
        object.insert("active".to_string(), JsonValue::Bool(active));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for payroll concept update",
        ));
    }

    Ok(JsonValue::Object(object))
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

pub type SurrealAnyPayrollConceptRepository = SurrealPayrollConceptRepository<Any>;
