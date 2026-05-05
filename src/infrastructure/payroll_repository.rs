use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::payroll::Payroll,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
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
        let _: Option<JsonValue> = self
            .client
            .create((PAYROLL_TABLE, id.to_string()))
            .content(json!({
                "name": name,
                "description": description,
                "organization_id": organization_id.to_string(),
            }))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created payroll"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Payroll>> {
        let rid = RecordId::new(PAYROLL_TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM payroll WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<PayrollRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>> {
        let mut response = self
            .client
            .query("SELECT * FROM payroll WHERE organization_id = $org_id")
            .bind(("org_id", organization_id.to_string()))
            .await?;

        let result: Result<Vec<PayrollRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Payroll>> {
        let payload = build_update_payload(name, description)?;

        let _: Option<JsonValue> = self
            .client
            .update((PAYROLL_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self.client.delete((PAYROLL_TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct PayrollRecord {
    id: RecordId,
    name: String,
    description: String,
    organization_id: String,
}

fn record_to_domain(record: PayrollRecord) -> AppResult<Payroll> {
    let id = parse_thing_id(&record.id.key, "stored payroll id")?;
    let organization_id =
        parse_uuid_field(&record.organization_id, "stored payroll organization_id")?;

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
