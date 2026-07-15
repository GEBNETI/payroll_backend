use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{
    engine::any::Any,
    types::{RecordId, SurrealValue},
    Connection, Surreal,
};
use uuid::Uuid;

use crate::{
    domain::user_role_assignment::UserRoleAssignment,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::user_role_assignment::{AssignRoleParams, UserRoleAssignmentRepository},
};

const ASSIGNMENT_TABLE: &str = "user_role_assignment";

// `?? NONE` coalesces SQL null to SurrealDB NONE so Option<String> deserializes correctly
const SELECT_FIELDS: &str = "SELECT id, user_id, role_id, role_name, \
     organization_id ?? NONE AS organization_id, \
     payroll_id ?? NONE AS payroll_id, \
     payroll_name ?? NONE AS payroll_name";

#[derive(Clone)]
pub struct SurrealUserRoleAssignmentRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealUserRoleAssignmentRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> UserRoleAssignmentRepository for SurrealUserRoleAssignmentRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, id: Uuid, params: AssignRoleParams) -> AppResult<UserRoleAssignment> {
        let mut payload = Map::new();
        payload.insert("user_id".into(), json!(params.user_id.to_string()));
        payload.insert("role_id".into(), json!(params.role_id.to_string()));
        payload.insert("role_name".into(), json!(params.role_name));
        if let Some(org_id) = params.organization_id {
            payload.insert("organization_id".into(), json!(org_id.to_string()));
        }
        if let Some(payroll_id) = params.payroll_id {
            payload.insert("payroll_id".into(), json!(payroll_id.to_string()));
        }
        if let Some(payroll_name) = params.payroll_name {
            payload.insert("payroll_name".into(), json!(payroll_name));
        }

        let _: Option<JsonValue> = self
            .client
            .create((ASSIGNMENT_TABLE, id.to_string()))
            .content(JsonValue::Object(payload))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created assignment"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<UserRoleAssignment>> {
        let rid = RecordId::new(ASSIGNMENT_TABLE, id.to_string());
        let query = format!("{SELECT_FIELDS} FROM user_role_assignment WHERE id = $rid");
        let mut response = self.client.query(query).bind(("rid", rid)).await?;

        let result: Result<Option<AssignmentRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_for_user(&self, user_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        let query = format!("{SELECT_FIELDS} FROM user_role_assignment WHERE user_id = $user_id");
        let mut response = self
            .client
            .query(query)
            .bind(("user_id", user_id.to_string()))
            .await?;

        let result: Result<Vec<AssignmentRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_for_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        let query =
            format!("{SELECT_FIELDS} FROM user_role_assignment WHERE payroll_id = $payroll_id");
        let mut response = self
            .client
            .query(query)
            .bind(("payroll_id", payroll_id.to_string()))
            .await?;

        let result: Result<Vec<AssignmentRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_for_org(&self, org_id: Uuid) -> AppResult<Vec<UserRoleAssignment>> {
        let query =
            format!("{SELECT_FIELDS} FROM user_role_assignment WHERE organization_id = $org_id");
        let mut response = self
            .client
            .query(query)
            .bind(("org_id", org_id.to_string()))
            .await?;

        let result: Result<Vec<AssignmentRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self
            .client
            .delete((ASSIGNMENT_TABLE, id.to_string()))
            .await?;
        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct AssignmentRecord {
    id: RecordId,
    user_id: String,
    role_id: String,
    role_name: String,
    organization_id: Option<String>,
    payroll_id: Option<String>,
    payroll_name: Option<String>,
}

fn record_to_domain(record: AssignmentRecord) -> AppResult<UserRoleAssignment> {
    let id = parse_thing_id(&record.id.key, "stored assignment id")?;
    let user_id = parse_uuid_field(&record.user_id, "user_id")?;
    let role_id = parse_uuid_field(&record.role_id, "role_id")?;
    let organization_id = record
        .organization_id
        .map(|s| parse_uuid_field(&s, "organization_id"))
        .transpose()?;
    let payroll_id = record
        .payroll_id
        .map(|s| parse_uuid_field(&s, "payroll_id"))
        .transpose()?;
    Ok(UserRoleAssignment::new(
        id,
        user_id,
        role_id,
        record.role_name,
        organization_id,
        payroll_id,
        record.payroll_name,
    ))
}

pub type SurrealAnyUserRoleAssignmentRepository = SurrealUserRoleAssignmentRepository<Any>;
