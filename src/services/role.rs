use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{domain::role::Role, error::AppResult};

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Role>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<Role>>;
    async fn fetch_by_name(&self, name: &str) -> AppResult<Option<Role>>;
    async fn fetch_all(&self) -> AppResult<Vec<Role>>;
}

#[derive(Clone)]
pub struct RoleService {
    repository: Arc<dyn RoleRepository>,
}

impl RoleService {
    pub fn new(repository: Arc<dyn RoleRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_or_create(&self, name: &str) -> AppResult<Role> {
        if let Some(role) = self.repository.fetch_by_name(name).await? {
            return Ok(role);
        }
        let id = Uuid::new_v4();
        self.repository.insert(id, name.to_string()).await
    }

    pub async fn get_by_name(&self, name: &str) -> AppResult<Option<Role>> {
        self.repository.fetch_by_name(name).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<Role>> {
        self.repository.fetch(id).await
    }

    pub async fn list(&self) -> AppResult<Vec<Role>> {
        self.repository.fetch_all().await
    }
}
