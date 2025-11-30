use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::payroll_concept_definition::PayrollConceptDefinition,
    error::{AppError, AppResult},
    services::payroll_concept_definition::PayrollConceptDefinitionRepository,
};

const PAYROLL_CONCEPT_DEFINITION_TABLE: &str = "payroll_concept_definition";

#[derive(Clone)]
pub struct SurrealPayrollConceptDefinitionRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealPayrollConceptDefinitionRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> PayrollConceptDefinitionRepository for SurrealPayrollConceptDefinitionRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(
        &self,
        id: Uuid,
        payroll_concept_id: Uuid,
        formula: String,
        condition: String,
    ) -> AppResult<PayrollConceptDefinition> {
        let record: Option<Record> = self
            .client
            .create((PAYROLL_CONCEPT_DEFINITION_TABLE, id.to_string()))
            .content(json!({
                "payroll_concept_id": payroll_concept_id,
                "formula": formula,
                "condition": condition,
            }))
            .await?;

        record.map(record_to_domain).transpose()?.ok_or_else(|| {
            AppError::internal("database did not return created payroll concept definition")
        })
    }

    async fn fetch_by_concept(
        &self,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        let records: Vec<Record> = self.client.select(PAYROLL_CONCEPT_DEFINITION_TABLE).await?;

        records
            .into_iter()
            .find(|record| record.payroll_concept_id == payroll_concept_id.to_string())
            .map(record_to_domain)
            .transpose()
    }

    async fn update(
        &self,
        id: Uuid,
        formula: Option<String>,
        condition: Option<String>,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        let payload = build_update_payload(formula, condition)?;
        let record: Option<Record> = self
            .client
            .update((PAYROLL_CONCEPT_DEFINITION_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<Record> = self
            .client
            .delete((PAYROLL_CONCEPT_DEFINITION_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    id: Thing,
    payroll_concept_id: String,
    formula: String,
    condition: String,
}

fn record_to_domain(record: Record) -> AppResult<PayrollConceptDefinition> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value).map_err(|_| {
            AppError::internal("stored payroll concept definition id is not a UUID")
        })?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored payroll concept definition identifier is not a supported format",
            ));
        }
    };

    let payroll_concept_id = Uuid::parse_str(&record.payroll_concept_id).map_err(|_| {
        AppError::internal("stored payroll concept definition concept id is not a UUID")
    })?;

    Ok(PayrollConceptDefinition::new(
        id,
        payroll_concept_id,
        record.formula,
        record.condition,
    ))
}

fn build_update_payload(
    formula: Option<String>,
    condition: Option<String>,
) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(formula) = formula {
        object.insert("formula".to_string(), JsonValue::String(formula));
    }

    if let Some(condition) = condition {
        object.insert("condition".to_string(), JsonValue::String(condition));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for payroll concept definition update",
        ));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyPayrollConceptDefinitionRepository =
    SurrealPayrollConceptDefinitionRepository<Any>;
