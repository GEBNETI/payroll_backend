use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct CreateOrganizationParams {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateOrganizationParams {
    pub name: Option<String>,
}

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Organization>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>>;
    async fn fetch_all(&self) -> AppResult<Vec<Organization>>;
    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Organization>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct OrganizationService {
    repository: Arc<dyn OrganizationRepository>,
}

impl OrganizationService {
    pub fn new(repository: Arc<dyn OrganizationRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, params: CreateOrganizationParams) -> AppResult<Organization> {
        let name = Self::normalize_name(&params.name)?;
        let id = Uuid::new_v4();
        self.repository.insert(id, name).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<Organization>> {
        self.repository.fetch(id).await
    }

    pub async fn list(&self) -> AppResult<Vec<Organization>> {
        let mut organizations = self.repository.fetch_all().await?;
        organizations.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(organizations)
    }

    pub async fn update(
        &self,
        id: Uuid,
        params: UpdateOrganizationParams,
    ) -> AppResult<Option<Organization>> {
        if params.name.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let name = params
            .name
            .as_deref()
            .map(Self::normalize_name)
            .transpose()?;

        self.repository.update(id, name).await
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    fn normalize_name(value: &str) -> AppResult<String> {
        let name = value.trim();
        if name.is_empty() {
            return Err(AppError::validation("organization name cannot be empty"));
        }

        Ok(name.to_string())
    }
}
