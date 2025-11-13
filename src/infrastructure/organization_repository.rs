use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
    services::organization::OrganizationRepository,
};

const ORGANIZATION_TABLE: &str = "organization";

#[derive(Clone)]
pub struct SurrealOrganizationRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealOrganizationRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> OrganizationRepository for SurrealOrganizationRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Organization> {
        let record: Option<OrganizationRecord> = self
            .client
            .create((ORGANIZATION_TABLE, id.to_string()))
            .content(json!({"name": name}))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created organization"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>> {
        let record: Option<OrganizationRecord> = self
            .client
            .select((ORGANIZATION_TABLE, id.to_string()))
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_all(&self) -> AppResult<Vec<Organization>> {
        let records: Vec<OrganizationRecord> = self.client.select(ORGANIZATION_TABLE).await?;
        records.into_iter().map(record_to_domain).collect()
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Organization>> {
        let payload = build_update_payload(name)?;

        let record: Option<OrganizationRecord> = self
            .client
            .update((ORGANIZATION_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<OrganizationRecord> = self
            .client
            .delete((ORGANIZATION_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct OrganizationRecord {
    id: Thing,
    name: String,
}

fn record_to_domain(record: OrganizationRecord) -> AppResult<Organization> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored organization id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored organization identifier is not a supported format",
            ));
        }
    };

    Ok(Organization::new(id, record.name))
}

pub type SurrealAnyOrganizationRepository = SurrealOrganizationRepository<Any>;

fn build_update_payload(name: Option<String>) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for organization update",
        ));
    }

    Ok(JsonValue::Object(object))
}
