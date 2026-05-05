use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{engine::any::Any, types::{RecordId, SurrealValue}, Connection, Surreal};
use uuid::Uuid;

use crate::{
    domain::job::Job,
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::job::JobRepository,
};

const JOB_TABLE: &str = "job";

#[derive(Clone)]
pub struct SurrealJobRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealJobRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> JobRepository for SurrealJobRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(
        &self,
        id: Uuid,
        job_title: String,
        salary: f64,
        payroll_id: Uuid,
    ) -> AppResult<Job> {
        let _: Option<JsonValue> = self
            .client
            .create((JOB_TABLE, id.to_string()))
            .content(json!({
                "job_title": job_title,
                "salary": salary,
                "payroll_id": payroll_id.to_string(),
            }))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created job"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Job>> {
        let rid = RecordId::new(JOB_TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM job WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<JobRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Job>> {
        let mut response = self
            .client
            .query("SELECT * FROM job WHERE payroll_id = $payroll_id")
            .bind(("payroll_id", payroll_id.to_string()))
            .await?;

        let result: Result<Vec<JobRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        job_title: Option<String>,
        salary: Option<f64>,
    ) -> AppResult<Option<Job>> {
        let payload = build_update_payload(job_title, salary)?;

        let _: Option<JsonValue> = self
            .client
            .update((JOB_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self
            .client
            .delete((JOB_TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct JobRecord {
    id: RecordId,
    job_title: String,
    salary: f64,
    payroll_id: String,
}

fn record_to_domain(record: JobRecord) -> AppResult<Job> {
    let id = parse_thing_id(&record.id.key, "stored job id")?;
    let payroll_id = parse_uuid_field(&record.payroll_id, "stored job payroll_id")?;

    Ok(Job::new(id, record.job_title, record.salary, payroll_id))
}

fn build_update_payload(job_title: Option<String>, salary: Option<f64>) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(job_title) = job_title {
        object.insert("job_title".to_string(), JsonValue::String(job_title));
    }
    if let Some(salary) = salary {
        object.insert("salary".to_string(), JsonValue::from(salary));
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for job update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyJobRepository = SurrealJobRepository<Any>;
