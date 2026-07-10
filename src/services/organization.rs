use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::organization::Organization,
    error::{AppError, AppResult},
};

const DEFAULT_RIF: &str = "G000000000";

#[derive(Debug, Clone)]
pub struct CreateOrganizationParams {
    pub name: String,
    pub rif: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateOrganizationParams {
    pub name: Option<String>,
    pub rif: Option<String>,
}

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn insert(&self, id: Uuid, name: String, rif: String) -> AppResult<Organization>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>>;
    async fn fetch_all(&self) -> AppResult<Vec<Organization>>;
    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        rif: Option<String>,
    ) -> AppResult<Option<Organization>>;
    async fn backfill_missing_rifs(&self, rif: String) -> AppResult<usize>;
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
        let rif = Self::normalize_rif(&params.rif)?;
        let id = Uuid::new_v4();
        self.repository.insert(id, name, rif).await
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
        if params.name.is_none() && params.rif.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let name = params
            .name
            .as_deref()
            .map(Self::normalize_name)
            .transpose()?;
        let rif = params
            .rif
            .as_deref()
            .map(Self::normalize_rif)
            .transpose()?;

        self.repository.update(id, name, rif).await
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    pub async fn backfill_missing_rifs(&self) -> AppResult<usize> {
        self.repository
            .backfill_missing_rifs(DEFAULT_RIF.to_string())
            .await
    }

    fn normalize_name(value: &str) -> AppResult<String> {
        let name = value.trim();
        if name.is_empty() {
            return Err(AppError::validation("organization name cannot be empty"));
        }

        Ok(name.to_string())
    }

    fn normalize_rif(value: &str) -> AppResult<String> {
        let rif = value.trim().to_ascii_uppercase();
        let mut characters = rif.chars();
        let Some(prefix) = characters.next() else {
            return Err(AppError::validation(
                "rif must start with G or J followed by 9 digits",
            ));
        };

        if !matches!(prefix, 'G' | 'J') || rif.len() != 10 || !characters.all(|value| value.is_ascii_digit()) {
            return Err(AppError::validation(
                "rif must start with G or J followed by 9 digits",
            ));
        }

        Ok(rif)
    }
}

#[cfg(test)]
mod tests {
    use super::OrganizationService;

    #[test]
    fn normalizes_and_validates_rifs() {
        assert_eq!(
            OrganizationService::normalize_rif(" g123456789 ").unwrap(),
            "G123456789"
        );
        assert_eq!(
            OrganizationService::normalize_rif("J000000000").unwrap(),
            "J000000000"
        );
        for invalid_rif in ["", "A123456789", "G12345678", "G1234567890", "G12345A789"] {
            assert!(OrganizationService::normalize_rif(invalid_rif).is_err());
        }
    }
}
