use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::role::Role,
    error::{parse_thing_id, AppError, AppResult},
    services::role::RoleRepository,
};

const ROLE_TABLE: &str = "app_role";

#[derive(Clone)]
pub struct SurrealRoleRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealRoleRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> RoleRepository for SurrealRoleRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Role> {
        let _: Option<JsonValue> = self
            .client
            .create((ROLE_TABLE, id.to_string()))
            .content(json!({"name": name}))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created role"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Role>> {
        let rid = RecordId::new(ROLE_TABLE, id.to_string());
        let mut response = self
            .client
            .query(format!("SELECT * FROM {ROLE_TABLE} WHERE id = $rid"))
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<RoleRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_name(&self, name: &str) -> AppResult<Option<Role>> {
        let mut response = self
            .client
            .query(format!("SELECT * FROM {ROLE_TABLE} WHERE name = $name LIMIT 1"))
            .bind(("name", name.to_string()))
            .await?;

        let result: Result<Vec<RoleRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().next().map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_all(&self) -> AppResult<Vec<Role>> {
        let mut response = self
            .client
            .query(format!("SELECT * FROM {ROLE_TABLE}"))
            .await?;

        let result: Result<Vec<RoleRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct RoleRecord {
    id: RecordId,
    name: String,
}

fn record_to_domain(record: RoleRecord) -> AppResult<Role> {
    let id = parse_thing_id(&record.id.key, "stored role id")?;
    Ok(Role::new(id, record.name))
}

pub type SurrealAnyRoleRepository = SurrealRoleRepository<Any>;
