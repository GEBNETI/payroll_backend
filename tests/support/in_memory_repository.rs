use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use nomina::{
    domain::{organization::Organization, payroll::Payroll},
    error::AppResult,
    services::{organization::OrganizationRepository, payroll::PayrollRepository},
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

#[derive(Default)]
pub struct InMemoryPayrollRepository {
    store: RwLock<HashMap<Uuid, Payroll>>,
}

#[async_trait]
impl PayrollRepository for InMemoryPayrollRepository {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        organization_id: Uuid,
    ) -> AppResult<Payroll> {
        let payroll = Payroll::new(id, name, description, organization_id);
        self.store.write().await.insert(payroll.id, payroll.clone());
        Ok(payroll)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Payroll>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_all(&self) -> AppResult<Vec<Payroll>> {
        Ok(self.store.read().await.values().cloned().collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        organization_id: Option<Uuid>,
    ) -> AppResult<Option<Payroll>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            if let Some(description) = description {
                existing.description = description;
            }
            if let Some(organization_id) = organization_id {
                existing.organization_id = organization_id;
            }

            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}
