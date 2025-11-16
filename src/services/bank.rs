use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::bank::Bank,
    error::{AppError, AppResult},
    services::organization::OrganizationService,
};

#[derive(Debug, Clone)]
pub struct CreateBankParams {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateBankParams {
    pub name: Option<String>,
}

#[async_trait]
pub trait BankRepository: Send + Sync {
    async fn insert(&self, id: Uuid, name: String, organization_id: Uuid) -> AppResult<Bank>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<Bank>>;
    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Bank>>;
    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Bank>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct BankService {
    repository: Arc<dyn BankRepository>,
    organization_service: Arc<OrganizationService>,
}

impl BankService {
    pub fn new(
        repository: Arc<dyn BankRepository>,
        organization_service: Arc<OrganizationService>,
    ) -> Self {
        Self {
            repository,
            organization_service,
        }
    }

    pub async fn create(&self, organization_id: Uuid, params: CreateBankParams) -> AppResult<Bank> {
        let name = Self::normalize_name(&params.name)?;
        self.ensure_organization_exists(organization_id).await?;
        let id = Uuid::new_v4();
        self.repository.insert(id, name, organization_id).await
    }

    pub async fn get(&self, organization_id: Uuid, bank_id: Uuid) -> AppResult<Option<Bank>> {
        let bank = self.repository.fetch(bank_id).await?;
        Ok(bank.filter(|bank| bank.organization_id == organization_id))
    }

    pub async fn list(&self, organization_id: Uuid) -> AppResult<Vec<Bank>> {
        self.ensure_organization_exists(organization_id).await?;
        let mut banks = self
            .repository
            .fetch_by_organization(organization_id)
            .await?;
        banks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(banks)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        bank_id: Uuid,
        params: UpdateBankParams,
    ) -> AppResult<Option<Bank>> {
        if params.name.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self.get(organization_id, bank_id).await?.is_none() {
            return Ok(None);
        }

        let name = params
            .name
            .as_deref()
            .map(Self::normalize_name)
            .transpose()?;

        self.repository.update(bank_id, name).await
    }

    pub async fn delete(&self, organization_id: Uuid, bank_id: Uuid) -> AppResult<bool> {
        if self.get(organization_id, bank_id).await?.is_none() {
            return Ok(false);
        }

        self.repository.delete(bank_id).await
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
            return Err(AppError::validation("bank name cannot be empty"));
        }

        Ok(name.to_string())
    }
}
