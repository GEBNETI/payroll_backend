use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::job::Job,
    error::{AppError, AppResult},
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
        let record: Option<JobRecord> = self
            .client
            .create((JOB_TABLE, id.to_string()))
            .content(json!({
                "job_title": job_title,
                "salary": salary,
                "payroll_id": payroll_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created job"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Job>> {
        let record: Option<JobRecord> = self.client.select((JOB_TABLE, id.to_string())).await?;
        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Job>> {
        let records: Vec<JobRecord> = self.client.select(JOB_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.payroll_id == payroll_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn update(
        &self,
        id: Uuid,
        job_title: Option<String>,
        salary: Option<f64>,
    ) -> AppResult<Option<Job>> {
        let payload = build_update_payload(job_title, salary)?;
        let record: Option<JobRecord> = self
            .client
            .update((JOB_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JobRecord> = self.client.delete((JOB_TABLE, id.to_string())).await?;
        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct JobRecord {
    id: Thing,
    job_title: String,
    salary: f64,
    payroll_id: String,
}

fn record_to_domain(record: JobRecord) -> AppResult<Job> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored job id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored job identifier is not a supported format",
            ));
        }
    };

    let payroll_id = Uuid::parse_str(&record.payroll_id)
        .map_err(|_| AppError::internal("stored job payroll id is not a UUID"))?;

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
