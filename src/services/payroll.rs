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
    pub organization_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePayrollParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
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

    async fn fetch_all(&self) -> AppResult<Vec<Payroll>>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        organization_id: Option<Uuid>,
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

    pub async fn create(&self, params: CreatePayrollParams) -> AppResult<Payroll> {
        let name = Self::normalize_name(&params.name)?;
        let description = Self::normalize_description(&params.description)?;
        self.ensure_organization_exists(params.organization_id)
            .await?;
        let id = Uuid::new_v4();
        self.repository
            .insert(id, name, description, params.organization_id)
            .await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<Payroll>> {
        self.repository.fetch(id).await
    }

    pub async fn list(&self) -> AppResult<Vec<Payroll>> {
        let mut payrolls = self.repository.fetch_all().await?;
        payrolls.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(payrolls)
    }

    pub async fn update(
        &self,
        id: Uuid,
        params: UpdatePayrollParams,
    ) -> AppResult<Option<Payroll>> {
        if params.name.is_none() && params.description.is_none() && params.organization_id.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if let Some(organization_id) = params.organization_id {
            self.ensure_organization_exists(organization_id).await?;
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

        self.repository
            .update(id, name, description, params.organization_id)
            .await
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
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
