use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use nomina::{
    domain::{division::Division, organization::Organization, payroll::Payroll},
    error::AppResult,
    services::{
        division::DivisionRepository, organization::OrganizationRepository,
        payroll::PayrollRepository,
    },
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

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|payroll| payroll.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Payroll>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            if let Some(description) = description {
                existing.description = description;
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
pub struct InMemoryDivisionRepository {
    store: RwLock<HashMap<Uuid, Division>>,
}

#[async_trait]
impl DivisionRepository for InMemoryDivisionRepository {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        budget_code: String,
        payroll_id: Uuid,
        parent_division_id: Option<Uuid>,
    ) -> AppResult<Division> {
        let division = Division::new(
            id,
            name,
            description,
            budget_code,
            payroll_id,
            parent_division_id,
        );
        self.store
            .write()
            .await
            .insert(division.id, division.clone());
        Ok(division)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Division>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Division>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|division| division.payroll_id == payroll_id)
            .cloned()
            .collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        budget_code: Option<String>,
        parent_division_id: Option<Option<Uuid>>,
    ) -> AppResult<Option<Division>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            if let Some(description) = description {
                existing.description = description;
            }
            if let Some(budget_code) = budget_code {
                existing.budget_code = budget_code;
            }
            if let Some(parent) = parent_division_id {
                existing.parent_division_id = parent;
            }

            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}
