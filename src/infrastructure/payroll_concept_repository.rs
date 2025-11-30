use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::payroll_concept::{
        PayrollConcept, PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType,
    },
    error::{AppError, AppResult},
    services::payroll_concept::PayrollConceptRepository,
};

const PAYROLL_CONCEPT_TABLE: &str = "payroll_concept";

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
    async fn insert(
        &self,
        id: Uuid,
        code: String,
        name: String,
        concept_type: PayrollConceptType,
        scope: PayrollConceptScope,
        period: PayrollConceptPeriod,
        active: bool,
        payroll_id: Uuid,
    ) -> AppResult<PayrollConcept> {
        let record: Option<PayrollConceptRecord> = self
            .client
            .create((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .content(json!({
                "code": code,
                "name": name,
                "type": concept_type,
                "scope": scope,
                "period": period,
                "active": active,
                "payroll_id": payroll_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created payroll concept"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollConcept>> {
        let record: Option<PayrollConceptRecord> = self
            .client
            .select((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .await?;
        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<PayrollConcept>> {
        let records: Vec<PayrollConceptRecord> = self.client.select(PAYROLL_CONCEPT_TABLE).await?;

        records
            .into_iter()
            .filter(|record| record.payroll_id == payroll_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn update(
        &self,
        id: Uuid,
        code: Option<String>,
        name: Option<String>,
        concept_type: Option<PayrollConceptType>,
        scope: Option<PayrollConceptScope>,
        period: Option<PayrollConceptPeriod>,
        active: Option<bool>,
    ) -> AppResult<Option<PayrollConcept>> {
        let payload = build_update_payload(code, name, concept_type, scope, period, active)?;
        let record: Option<PayrollConceptRecord> = self
            .client
            .update((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<PayrollConceptRecord> = self
            .client
            .delete((PAYROLL_CONCEPT_TABLE, id.to_string()))
            .await?;
        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct PayrollConceptRecord {
    id: Thing,
    code: String,
    name: String,
    #[serde(rename = "type")]
    concept_type: PayrollConceptType,
    scope: PayrollConceptScope,
    period: PayrollConceptPeriod,
    active: bool,
    payroll_id: String,
}

fn record_to_domain(record: PayrollConceptRecord) -> AppResult<PayrollConcept> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored payroll concept id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored payroll concept identifier is not a supported format",
            ));
        }
    };

    let payroll_id = Uuid::parse_str(&record.payroll_id)
        .map_err(|_| AppError::internal("stored payroll concept payroll id is not a UUID"))?;

    Ok(PayrollConcept::new(
        id,
        record.code,
        record.name,
        record.concept_type,
        record.scope,
        record.period,
        record.active,
        payroll_id,
    ))
}

fn build_update_payload(
    code: Option<String>,
    name: Option<String>,
    concept_type: Option<PayrollConceptType>,
    scope: Option<PayrollConceptScope>,
    period: Option<PayrollConceptPeriod>,
    active: Option<bool>,
) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(code) = code {
        object.insert("code".to_string(), JsonValue::String(code));
    }

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }

    if let Some(concept_type) = concept_type {
        let value = serde_json::to_value(concept_type).map_err(|_| {
            AppError::internal("failed to serialize concept type for update payload")
        })?;
        object.insert("type".to_string(), value);
    }

    if let Some(scope) = scope {
        let value = serde_json::to_value(scope).map_err(|_| {
            AppError::internal("failed to serialize concept scope for update payload")
        })?;
        object.insert("scope".to_string(), value);
    }

    if let Some(period) = period {
        let value = serde_json::to_value(period).map_err(|_| {
            AppError::internal("failed to serialize concept period for update payload")
        })?;
        object.insert("period".to_string(), value);
    }

    if let Some(active) = active {
        object.insert("active".to_string(), JsonValue::Bool(active));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for payroll concept update",
        ));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyPayrollConceptRepository = SurrealPayrollConceptRepository<Any>;
