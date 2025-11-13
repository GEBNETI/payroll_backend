use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use nomina::{
    domain::organization::Organization, error::AppResult,
    services::organization::OrganizationRepository,
};

#[derive(Default)]
pub struct InMemoryOrganizationRepository {
    store: RwLock<HashMap<Uuid, Organization>>,
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Organization> {
        let organization = Organization::new(id, name);
        self.store
            .write()
            .await
            .insert(organization.id, organization.clone());
        Ok(organization)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_all(&self) -> AppResult<Vec<Organization>> {
        Ok(self.store.read().await.values().cloned().collect())
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Organization>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}
