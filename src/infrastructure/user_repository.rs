use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::user::User,
    error::{parse_thing_id, AppError, AppResult},
    services::user::{InsertUserParams, UpdateUserParams, UserRepository},
};

const USER_TABLE: &str = "app_user";

#[derive(Clone)]
pub struct SurrealUserRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealUserRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> UserRepository for SurrealUserRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, id: Uuid, params: InsertUserParams) -> AppResult<User> {
        let _: Option<JsonValue> = self
            .client
            .create((USER_TABLE, id.to_string()))
            .content(json!({
                "username": params.username,
                "email": params.email,
                "password_hash": params.password_hash,
                "name": params.name,
                "is_active": true,
            }))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created user"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<User>> {
        let rid = RecordId::new(USER_TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM app_user WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<UserRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let mut response = self
            .client
            .query("SELECT * FROM app_user WHERE username = $username LIMIT 1")
            .bind(("username", username.to_string()))
            .await?;

        let result: Result<Vec<UserRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().next().map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_all(&self) -> AppResult<Vec<User>> {
        let mut response = self.client.query("SELECT * FROM app_user").await?;

        let result: Result<Vec<UserRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(&self, id: Uuid, params: UpdateUserParams) -> AppResult<Option<User>> {
        let payload = build_update_payload(params)?;

        let _: Option<JsonValue> = self
            .client
            .update((USER_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self.client.delete((USER_TABLE, id.to_string())).await?;
        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct UserRecord {
    id: RecordId,
    username: String,
    email: String,
    password_hash: String,
    name: String,
    is_active: bool,
}

fn record_to_domain(record: UserRecord) -> AppResult<User> {
    let id = parse_thing_id(&record.id.key, "stored user id")?;
    Ok(User::new(
        id,
        record.username,
        record.email,
        record.password_hash,
        record.name,
        record.is_active,
    ))
}

fn build_update_payload(params: UpdateUserParams) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(email) = params.email {
        object.insert("email".to_string(), JsonValue::String(email));
    }
    if let Some(name) = params.name {
        object.insert("name".to_string(), JsonValue::String(name));
    }
    if let Some(password_hash) = params.password_hash {
        object.insert(
            "password_hash".to_string(),
            JsonValue::String(password_hash),
        );
    }
    if let Some(is_active) = params.is_active {
        object.insert("is_active".to_string(), JsonValue::Bool(is_active));
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for user update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyUserRepository = SurrealUserRepository<Any>;
