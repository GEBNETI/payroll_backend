use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::job::Job,
    error::{AppError, AppResult},
    services::payroll::PayrollService,
};

#[derive(Debug, Clone)]
pub struct CreateJobParams {
    pub job_title: String,
    pub salary: f64,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateJobParams {
    pub job_title: Option<String>,
    pub salary: Option<f64>,
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn insert(
        &self,
        id: Uuid,
        job_title: String,
        salary: f64,
        payroll_id: Uuid,
    ) -> AppResult<Job>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Job>>;

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Job>>;

    async fn update(
        &self,
        id: Uuid,
        job_title: Option<String>,
        salary: Option<f64>,
    ) -> AppResult<Option<Job>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct JobService {
    repository: Arc<dyn JobRepository>,
    payroll_service: Arc<PayrollService>,
}

impl JobService {
    pub fn new(repository: Arc<dyn JobRepository>, payroll_service: Arc<PayrollService>) -> Self {
        Self {
            repository,
            payroll_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        params: CreateJobParams,
    ) -> AppResult<Job> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let job_title = Self::normalize_title(&params.job_title)?;
        let salary = Self::validate_salary(params.salary)?;
        let id = Uuid::new_v4();

        self.repository
            .insert(id, job_title, salary, payroll_id)
            .await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        job_id: Uuid,
    ) -> AppResult<Option<Job>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let job = self.repository.fetch(job_id).await?;
        Ok(job.filter(|job| job.payroll_id == payroll_id))
    }

    pub async fn list(&self, organization_id: Uuid, payroll_id: Uuid) -> AppResult<Vec<Job>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let mut jobs = self.repository.fetch_by_payroll(payroll_id).await?;
        jobs.sort_by(|a, b| a.job_title.cmp(&b.job_title));
        Ok(jobs)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        job_id: Uuid,
        params: UpdateJobParams,
    ) -> AppResult<Option<Job>> {
        if params.job_title.is_none() && params.salary.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self
            .get(organization_id, payroll_id, job_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }

        let job_title = params
            .job_title
            .as_deref()
            .map(Self::normalize_title)
            .transpose()?;
        let salary = params.salary.map(Self::validate_salary).transpose()?;

        self.repository.update(job_id, job_title, salary).await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        job_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, job_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(job_id).await
    }

    async fn ensure_payroll_accessible(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<()> {
        self.payroll_service
            .ensure_belongs_to_organization(organization_id, payroll_id)
            .await
    }

    fn normalize_title(value: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation("job title cannot be empty"));
        }

        Ok(trimmed.to_string())
    }

    fn validate_salary(value: f64) -> AppResult<f64> {
        if value <= 0.0 {
            return Err(AppError::validation("salary must be greater than zero"));
        }

        Ok(value)
    }
}
