use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::payroll::Payroll,
    error::{AppError, AppResult},
    services::organization::OrganizationService,
};

#[derive(Debug, Clone)]
pub struct CreatePayrollParams {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePayrollParams {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[async_trait]
pub trait PayrollRepository: Send + Sync {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        organization_id: Uuid,
    ) -> AppResult<Payroll>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Payroll>>;

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Payroll>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PayrollService {
    repository: Arc<dyn PayrollRepository>,
    organization_service: Arc<OrganizationService>,
}

impl PayrollService {
    pub fn new(
        repository: Arc<dyn PayrollRepository>,
        organization_service: Arc<OrganizationService>,
    ) -> Self {
        Self {
            repository,
            organization_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        params: CreatePayrollParams,
    ) -> AppResult<Payroll> {
        let name = Self::normalize_name(&params.name)?;
        let description = Self::normalize_description(&params.description)?;
        self.ensure_organization_exists(organization_id).await?;
        let id = Uuid::new_v4();
        self.repository
            .insert(id, name, description, organization_id)
            .await
    }

    pub async fn get(&self, organization_id: Uuid, payroll_id: Uuid) -> AppResult<Option<Payroll>> {
        let payroll = self.repository.fetch(payroll_id).await?;
        Ok(payroll.filter(|payroll| payroll.organization_id == organization_id))
    }

    pub async fn list(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>> {
        self.ensure_organization_exists(organization_id).await?;
        let mut payrolls = self
            .repository
            .fetch_by_organization(organization_id)
            .await?;
        payrolls.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(payrolls)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        params: UpdatePayrollParams,
    ) -> AppResult<Option<Payroll>> {
        if params.name.is_none() && params.description.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self.get(organization_id, payroll_id).await?.is_none() {
            return Ok(None);
        }

        let name = params
            .name
            .as_deref()
            .map(Self::normalize_name)
            .transpose()?;
        let description = params
            .description
            .as_deref()
            .map(Self::normalize_description)
            .transpose()?;

        self.repository.update(payroll_id, name, description).await
    }

    pub async fn delete(&self, organization_id: Uuid, payroll_id: Uuid) -> AppResult<bool> {
        if self.get(organization_id, payroll_id).await?.is_none() {
            return Ok(false);
        }

        self.repository.delete(payroll_id).await
    }

    pub async fn ensure_belongs_to_organization(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<()> {
        if self.get(organization_id, payroll_id).await?.is_some() {
            Ok(())
        } else {
            Err(AppError::not_found(format!(
                "payroll `{payroll_id}` not found for organization `{organization_id}`"
            )))
        }
    }

    async fn ensure_organization_exists(&self, organization_id: Uuid) -> AppResult<()> {
        let exists = self
            .organization_service
            .get(organization_id)
            .await?
            .is_some();

        if exists {
            Ok(())
        } else {
            Err(AppError::not_found(format!(
                "organization `{organization_id}` not found"
            )))
        }
    }

    fn normalize_name(value: &str) -> AppResult<String> {
        let name = value.trim();
        if name.is_empty() {
            return Err(AppError::validation("payroll name cannot be empty"));
        }

        Ok(name.to_string())
    }

    fn normalize_description(value: &str) -> AppResult<String> {
        let description = value.trim();
        if description.is_empty() {
            return Err(AppError::validation("payroll description cannot be empty"));
        }

        Ok(description.to_string())
    }
}
