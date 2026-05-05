use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::bank::Bank,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::bank::BankRepository,
};

const BANK_TABLE: &str = "bank";

#[derive(Clone)]
pub struct SurrealBankRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealBankRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> BankRepository for SurrealBankRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, id: Uuid, name: String, organization_id: Uuid) -> AppResult<Bank> {
        let _: Option<JsonValue> = self
            .client
            .create((BANK_TABLE, id.to_string()))
            .content(json!({
                "name": name,
                "organization_id": organization_id.to_string(),
            }))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created bank"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Bank>> {
        let rid = RecordId::new(BANK_TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM bank WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<BankRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Bank>> {
        let mut response = self
            .client
            .query("SELECT * FROM bank WHERE organization_id = $org_id")
            .bind(("org_id", organization_id.to_string()))
            .await?;

        let result: Result<Vec<BankRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Bank>> {
        let payload = build_update_payload(name)?;

        let _: Option<JsonValue> = self
            .client
            .update((BANK_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self.client.delete((BANK_TABLE, id.to_string())).await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct BankRecord {
    id: RecordId,
    name: String,
    organization_id: String,
}

fn record_to_domain(record: BankRecord) -> AppResult<Bank> {
    let id = parse_thing_id(&record.id.key, "stored bank id")?;
    let organization_id = parse_uuid_field(&record.organization_id, "stored bank organization_id")?;

    Ok(Bank::new(id, record.name, organization_id))
}

fn build_update_payload(name: Option<String>) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for bank update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyBankRepository = SurrealBankRepository<Any>;
