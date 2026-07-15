use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{parse_thing_id, AppError, AppResult},
    services::organization::OrganizationRepository,
};

const ORGANIZATION_TABLE: &str = "organization";
const SELECT_ORGANIZATION_FIELDS: &str = "SELECT id, name, rif ?? NONE AS rif";

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
    async fn insert(&self, id: Uuid, name: String, rif: String) -> AppResult<Organization> {
        let _: Option<JsonValue> = self
            .client
            .create((ORGANIZATION_TABLE, id.to_string()))
            .content(json!({"name": name, "rif": rif}))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created organization"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>> {
        let rid = RecordId::new(ORGANIZATION_TABLE, id.to_string());
        let mut response = self
            .client
            .query(format!(
                "{SELECT_ORGANIZATION_FIELDS} FROM organization WHERE id = $rid"
            ))
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<OrganizationRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_all(&self) -> AppResult<Vec<Organization>> {
        let mut response = self
            .client
            .query(format!("{SELECT_ORGANIZATION_FIELDS} FROM organization"))
            .await?;

        let result: Result<Vec<OrganizationRecord>, _> = response.take(0);
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
        rif: Option<String>,
    ) -> AppResult<Option<Organization>> {
        let payload = build_update_payload(name, rif)?;

        let _: Option<JsonValue> = self
            .client
            .update((ORGANIZATION_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self
            .client
            .delete((ORGANIZATION_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }

    async fn backfill_missing_rifs(&self, rif: String) -> AppResult<usize> {
        let mut response = self
            .client
            .query("UPDATE organization SET rif = $rif WHERE !rif")
            .bind(("rif", rif))
            .await?;
        let updated: Vec<JsonValue> = response.take(0)?;
        Ok(updated.len())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct OrganizationRecord {
    id: RecordId,
    name: String,
    rif: Option<String>,
}

fn record_to_domain(record: OrganizationRecord) -> AppResult<Organization> {
    let id = parse_thing_id(&record.id.key, "stored organization id")?;
    Ok(Organization::new(
        id,
        record.name,
        record.rif.unwrap_or_else(|| "G000000000".to_string()),
    ))
}

fn build_update_payload(name: Option<String>, rif: Option<String>) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(name) = name {
        object.insert("name".to_string(), JsonValue::String(name));
    }
    if let Some(rif) = rif {
        object.insert("rif".to_string(), JsonValue::String(rif));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for organization update",
        ));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyOrganizationRepository = SurrealOrganizationRepository<Any>;
