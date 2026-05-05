use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::payroll_concept_definition::PayrollConceptDefinition,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::payroll_concept_definition::PayrollConceptDefinitionRepository,
};

const TABLE: &str = "payroll_concept_definition";

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
        let _: Option<JsonValue> = self
            .client
            .create((TABLE, id.to_string()))
            .content(json!({
                "payroll_concept_id": payroll_concept_id.to_string(),
                "formula": formula,
                "condition": condition,
            }))
            .await?;

        self.fetch_by_concept(payroll_concept_id)
            .await?
            .ok_or_else(|| {
                AppError::internal("database did not return created payroll concept definition")
            })
    }

    async fn fetch_by_concept(
        &self,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        let mut response = self
            .client
            .query("SELECT * FROM payroll_concept_definition WHERE payroll_concept_id = $id")
            .bind(("id", payroll_concept_id.to_string()))
            .await?;

        // In SurrealDB v3 schemaless mode, querying a table with no rows yet returns a
        // per-statement "table does not exist" error instead of an empty result.
        let result: Result<Option<Record>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        formula: Option<String>,
        condition: Option<String>,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        let payload = build_update_payload(formula, condition)?;

        let _: Option<JsonValue> = self
            .client
            .update((TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self.client.delete((TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

impl<C> SurrealPayrollConceptDefinitionRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn fetch_by_id(&self, id: Uuid) -> AppResult<Option<PayrollConceptDefinition>> {
        let rid = RecordId::new(TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM payroll_concept_definition WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<Record>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct Record {
    id: RecordId,
    payroll_concept_id: String,
    formula: String,
    condition: String,
}

fn record_to_domain(record: Record) -> AppResult<PayrollConceptDefinition> {
    let id = parse_thing_id(&record.id.key, "stored payroll concept definition id")?;
    let payroll_concept_id = parse_uuid_field(
        &record.payroll_concept_id,
        "stored payroll concept definition payroll_concept_id",
    )?;

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
