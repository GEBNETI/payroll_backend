use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::division::Division,
    error::{AppError, AppResult},
    services::payroll::PayrollService,
};

#[derive(Debug, Clone)]
pub struct CreateDivisionParams {
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub parent_division_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateDivisionParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub budget_code: Option<String>,
    pub parent_division_id: Option<Option<Uuid>>,
}

#[async_trait]
pub trait DivisionRepository: Send + Sync {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        budget_code: String,
        payroll_id: Uuid,
        parent_division_id: Option<Uuid>,
    ) -> AppResult<Division>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Division>>;

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Division>>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        budget_code: Option<String>,
        parent_division_id: Option<Option<Uuid>>,
    ) -> AppResult<Option<Division>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct DivisionService {
    repository: Arc<dyn DivisionRepository>,
    payroll_service: Arc<PayrollService>,
}

impl DivisionService {
    pub fn new(
        repository: Arc<dyn DivisionRepository>,
        payroll_service: Arc<PayrollService>,
    ) -> Self {
        Self {
            repository,
            payroll_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        params: CreateDivisionParams,
    ) -> AppResult<Division> {
        let name = Self::normalize_field(&params.name, "division name")?;
        let description = Self::normalize_field(&params.description, "division description")?;
        let budget_code = Self::normalize_field(&params.budget_code, "division budget code")?;
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let parent_division_id = self
            .validate_parent(params.parent_division_id, payroll_id, None)
            .await?;

        let id = Uuid::new_v4();
        self.repository
            .insert(
                id,
                name,
                description,
                budget_code,
                payroll_id,
                parent_division_id,
            )
            .await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
    ) -> AppResult<Option<Division>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let division = self.repository.fetch(division_id).await?;
        Ok(division.filter(|division| division.payroll_id == payroll_id))
    }

    pub async fn list(&self, organization_id: Uuid, payroll_id: Uuid) -> AppResult<Vec<Division>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;
        let mut divisions = self.repository.fetch_by_payroll(payroll_id).await?;
        divisions.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(divisions)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        params: UpdateDivisionParams,
    ) -> AppResult<Option<Division>> {
        if params.name.is_none()
            && params.description.is_none()
            && params.budget_code.is_none()
            && params.parent_division_id.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self
            .get(organization_id, payroll_id, division_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }

        let parent_update = match params.parent_division_id {
            Some(parent_field) => Some(
                self.validate_parent(parent_field, payroll_id, Some(division_id))
                    .await?,
            ),
            None => None,
        };

        let name = params
            .name
            .as_deref()
            .map(|value| Self::normalize_field(value, "division name"))
            .transpose()?;
        let description = params
            .description
            .as_deref()
            .map(|value| Self::normalize_field(value, "division description"))
            .transpose()?;
        let budget_code = params
            .budget_code
            .as_deref()
            .map(|value| Self::normalize_field(value, "division budget code"))
            .transpose()?;

        self.repository
            .update(division_id, name, description, budget_code, parent_update)
            .await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, division_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(division_id).await
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

    async fn validate_parent(
        &self,
        parent_division_id: Option<Uuid>,
        target_payroll_id: Uuid,
        division_id: Option<Uuid>,
    ) -> AppResult<Option<Uuid>> {
        if let Some(parent_id) = parent_division_id {
            if Some(parent_id) == division_id {
                return Err(AppError::validation("division cannot be its own parent"));
            }

            let parent = self.repository.fetch(parent_id).await?.ok_or_else(|| {
                AppError::not_found(format!("parent division `{parent_id}` not found"))
            })?;

            if parent.payroll_id != target_payroll_id {
                return Err(AppError::validation(
                    "parent division must belong to the same payroll",
                ));
            }

            Ok(Some(parent_id))
        } else {
            Ok(None)
        }
    }

    fn normalize_field(value: &str, field: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(format!("{field} cannot be empty")));
        }

        Ok(trimmed.to_string())
    }
}
