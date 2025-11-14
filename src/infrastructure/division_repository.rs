use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::division::Division,
    error::{AppError, AppResult},
    services::division::DivisionRepository,
};

const DIVISION_TABLE: &str = "division";

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
        let record: Option<DivisionRecord> = self
            .client
            .create((DIVISION_TABLE, id.to_string()))
            .content(json!({
                "name": name,
                "description": description,
                "budget_code": budget_code,
                "payroll_id": payroll_id,
                "parent_division_id": parent_division_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created division"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Division>> {
        let record: Option<DivisionRecord> =
            self.client.select((DIVISION_TABLE, id.to_string())).await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Division>> {
        let records: Vec<DivisionRecord> = self.client.select(DIVISION_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.payroll_id == payroll_id.to_string())
            .map(record_to_domain)
            .collect()
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

        let record: Option<DivisionRecord> = self
            .client
            .update((DIVISION_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<DivisionRecord> =
            self.client.delete((DIVISION_TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct DivisionRecord {
    id: Thing,
    name: String,
    description: String,
    budget_code: String,
    payroll_id: String,
    parent_division_id: Option<String>,
}

fn record_to_domain(record: DivisionRecord) -> AppResult<Division> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored division id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored division identifier is not a supported format",
            ));
        }
    };

    let payroll_id = Uuid::parse_str(&record.payroll_id)
        .map_err(|_| AppError::internal("stored division payroll id is not a UUID"))?;
    let parent_division_id = match record.parent_division_id {
        Some(value) => Some(
            Uuid::parse_str(&value)
                .map_err(|_| AppError::internal("stored parent division id is not a UUID"))?,
        ),
        None => None,
    };

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
