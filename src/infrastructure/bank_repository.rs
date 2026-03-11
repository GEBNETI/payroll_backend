use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{engine::any::Any, types::{RecordId, SurrealValue}, Connection, Surreal};
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
        let record: Option<BankRecord> = self
            .client
            .create((BANK_TABLE, id.to_string()))
            .content(json!({
                "name": name,
                "organization_id": organization_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created bank"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Bank>> {
        let record: Option<BankRecord> = self.client.select((BANK_TABLE, id.to_string())).await?;
        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Bank>> {
        let records: Vec<BankRecord> = self.client.select(BANK_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.organization_id == organization_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Bank>> {
        let payload = build_update_payload(name)?;
        let record: Option<BankRecord> = self
            .client
            .update((BANK_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<BankRecord> = self.client.delete((BANK_TABLE, id.to_string())).await?;
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
