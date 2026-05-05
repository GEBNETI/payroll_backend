use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::division::Division,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::division::DivisionRepository,
};

const DIVISION_TABLE: &str = "division";

// `??` coalesces null/NONE to the right-hand side, normalising stored SQL null
// to NONE so SurrealValue deserialization of Option<String> succeeds.
const SELECT_DIVISION_FIELDS: &str = "SELECT id, name, description, budget_code, payroll_id, \
     parent_division_id ?? NONE AS parent_division_id";

#[derive(Clone)]
pub struct SurrealDivisionRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealDivisionRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> DivisionRepository for SurrealDivisionRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        budget_code: String,
        payroll_id: Uuid,
        parent_division_id: Option<Uuid>,
    ) -> AppResult<Division> {
        let mut payload = Map::new();
        payload.insert("name".to_string(), json!(name));
        payload.insert("description".to_string(), json!(description));
        payload.insert("budget_code".to_string(), json!(budget_code));
        payload.insert("payroll_id".to_string(), json!(payroll_id.to_string()));
        if let Some(parent_id) = parent_division_id {
            payload.insert(
                "parent_division_id".to_string(),
                json!(parent_id.to_string()),
            );
        }

        // Discard the create return value (surrealdb v3 returns null for absent optional
        // fields) and re-fetch through the null-safe query instead.
        let _: Option<JsonValue> = self
            .client
            .create((DIVISION_TABLE, id.to_string()))
            .content(JsonValue::Object(payload))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created division"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Division>> {
        let rid = RecordId::new(DIVISION_TABLE, id.to_string());
        let mut response = self
            .client
            .query(format!(
                "{SELECT_DIVISION_FIELDS} FROM division WHERE id = $rid"
            ))
            .bind(("rid", rid))
            .await?;

        let record: Option<DivisionRecord> = response.take(0)?;
        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Division>> {
        let mut response = self
            .client
            .query(format!(
                "{SELECT_DIVISION_FIELDS} FROM division WHERE payroll_id = $payroll_id"
            ))
            .bind(("payroll_id", payroll_id.to_string()))
            .await?;

        let records: Vec<DivisionRecord> = response.take(0)?;
        records.into_iter().map(record_to_domain).collect()
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        budget_code: Option<String>,
        parent_division_id: Option<Option<Uuid>>,
    ) -> AppResult<Option<Division>> {
        let payload = build_update_payload(name, description, budget_code, parent_division_id)?;

        // Discard the merge return value to avoid SurrealValue null-deserialization issues,
        // then re-fetch with the null-safe query. Returns None when the record doesn't exist.
        let _: Option<JsonValue> = self
            .client
            .update((DIVISION_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        // Use JsonValue to avoid SurrealValue deserialization issues with null fields
        // in the returned record when the record has optional fields stored as null.
        let record: Option<JsonValue> =
            self.client.delete((DIVISION_TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DivisionRecord {
    id: RecordId,
    name: String,
    description: String,
    budget_code: String,
    payroll_id: String,
    parent_division_id: Option<String>,
}

fn record_to_domain(record: DivisionRecord) -> AppResult<Division> {
    let id = parse_thing_id(&record.id.key, "stored division id")?;
    let payroll_id = parse_uuid_field(&record.payroll_id, "stored division payroll_id")?;
    let parent_division_id = record
        .parent_division_id
        .as_deref()
        .map(|v| parse_uuid_field(v, "stored division parent_division_id"))
        .transpose()?;

    Ok(Division::new(
        id,
        record.name,
        record.description,
        record.budget_code,
        payroll_id,
        parent_division_id,
    ))
}

fn build_update_payload(
    name: Option<String>,
    description: Option<String>,
    budget_code: Option<String>,
    parent_division_id: Option<Option<Uuid>>,
) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }

    if let Some(description) = description {
        object.insert("description".to_string(), JsonValue::String(description));
    }

    if let Some(budget_code) = budget_code {
        object.insert("budget_code".to_string(), JsonValue::String(budget_code));
    }

    if let Some(parent) = parent_division_id {
        match parent {
            Some(value) => {
                object.insert(
                    "parent_division_id".to_string(),
                    JsonValue::String(value.to_string()),
                );
            }
            None => {
                object.insert("parent_division_id".to_string(), JsonValue::Null);
            }
        }
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for division update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyDivisionRepository = SurrealDivisionRepository<Any>;
