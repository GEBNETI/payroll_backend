use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::payroll::Payroll,
    error::{AppError, AppResult},
    services::payroll::PayrollRepository,
};

const PAYROLL_TABLE: &str = "payroll";

#[derive(Clone)]
pub struct SurrealPayrollRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealPayrollRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> PayrollRepository for SurrealPayrollRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        organization_id: Uuid,
    ) -> AppResult<Payroll> {
        let record: Option<PayrollRecord> = self
            .client
            .create((PAYROLL_TABLE, id.to_string()))
            .content(json!({
                "name": name,
                "description": description,
                "organization_id": organization_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created payroll"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Payroll>> {
        let record: Option<PayrollRecord> =
            self.client.select((PAYROLL_TABLE, id.to_string())).await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>> {
        let records: Vec<PayrollRecord> = self.client.select(PAYROLL_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.organization_id == organization_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Payroll>> {
        let payload = build_update_payload(name, description)?;
        let record: Option<PayrollRecord> = self
            .client
            .update((PAYROLL_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<PayrollRecord> =
            self.client.delete((PAYROLL_TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct PayrollRecord {
    id: Thing,
    name: String,
    description: String,
    organization_id: String,
}

fn record_to_domain(record: PayrollRecord) -> AppResult<Payroll> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored payroll id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored payroll identifier is not a supported format",
            ));
        }
    };

    let organization_id = Uuid::parse_str(&record.organization_id)
        .map_err(|_| AppError::internal("stored payroll organization id is not a UUID"))?;

    Ok(Payroll::new(
        id,
        record.name,
        record.description,
        organization_id,
    ))
}

fn build_update_payload(name: Option<String>, description: Option<String>) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }

    if let Some(description) = description {
        object.insert("description".to_string(), JsonValue::String(description));
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for payroll update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyPayrollRepository = SurrealPayrollRepository<Any>;
