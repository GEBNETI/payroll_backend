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
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateDivisionParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub budget_code: Option<String>,
    pub payroll_id: Option<Uuid>,
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

    async fn fetch_all(&self) -> AppResult<Vec<Division>>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        budget_code: Option<String>,
        payroll_id: Option<Uuid>,
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

    pub async fn create(&self, params: CreateDivisionParams) -> AppResult<Division> {
        let name = Self::normalize_field(&params.name, "division name")?;
        let description = Self::normalize_field(&params.description, "division description")?;
        let budget_code = Self::normalize_field(&params.budget_code, "division budget code")?;
        self.ensure_payroll_exists(params.payroll_id).await?;
        let parent_division_id = self
            .validate_parent(params.parent_division_id, Some(params.payroll_id), None)
            .await?;

        let id = Uuid::new_v4();
        self.repository
            .insert(
                id,
                name,
                description,
                budget_code,
                params.payroll_id,
                parent_division_id,
            )
            .await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<Division>> {
        self.repository.fetch(id).await
    }

    pub async fn list(&self) -> AppResult<Vec<Division>> {
        let mut divisions = self.repository.fetch_all().await?;
        divisions.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(divisions)
    }

    pub async fn update(
        &self,
        id: Uuid,
        params: UpdateDivisionParams,
    ) -> AppResult<Option<Division>> {
        if params.name.is_none()
            && params.description.is_none()
            && params.budget_code.is_none()
            && params.payroll_id.is_none()
            && params.parent_division_id.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let existing = match self.repository.fetch(id).await? {
            Some(division) => division,
            None => return Ok(None),
        };

        if let Some(payroll_id) = params.payroll_id {
            self.ensure_payroll_exists(payroll_id).await?;
        }

        let target_payroll_id = params.payroll_id.unwrap_or(existing.payroll_id);

        if params.parent_division_id.is_none()
            && params.payroll_id.is_some()
            && let Some(current_parent) = existing.parent_division_id
        {
            self.validate_parent(Some(current_parent), Some(target_payroll_id), Some(id))
                .await?;
        }

        let parent_update = match params.parent_division_id {
            Some(parent_field) => Some(
                self.validate_parent(parent_field, Some(target_payroll_id), Some(id))
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
            .update(
                id,
                name,
                description,
                budget_code,
                params.payroll_id,
                parent_update,
            )
            .await
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    async fn ensure_payroll_exists(&self, payroll_id: Uuid) -> AppResult<()> {
        let exists = self.payroll_service.get(payroll_id).await?.is_some();
        if exists {
            Ok(())
        } else {
            Err(AppError::not_found(format!(
                "payroll `{payroll_id}` not found"
            )))
        }
    }

    async fn validate_parent(
        &self,
        parent_division_id: Option<Uuid>,
        target_payroll_id: Option<Uuid>,
        division_id: Option<Uuid>,
    ) -> AppResult<Option<Uuid>> {
        if let Some(parent_id) = parent_division_id {
            if Some(parent_id) == division_id {
                return Err(AppError::validation("division cannot be its own parent"));
            }

            let parent = self.repository.fetch(parent_id).await?.ok_or_else(|| {
                AppError::not_found(format!("parent division `{parent_id}` not found"))
            })?;

            if let Some(target_payroll) = target_payroll_id
                && parent.payroll_id != target_payroll
            {
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
