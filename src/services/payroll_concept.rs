use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::payroll_concept::{
        PayrollConcept, PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType,
    },
    error::{AppError, AppResult},
    services::payroll::PayrollService,
};

/// Parameters for creating a new payroll concept (used by service layer).
#[derive(Debug, Clone)]
pub struct CreatePayrollConceptParams {
    pub code: String,
    pub name: String,
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
    pub period: PayrollConceptPeriod,
    pub active: bool,
}

/// Parameters for updating an existing payroll concept.
#[derive(Debug, Clone, Default)]
pub struct UpdatePayrollConceptParams {
    pub code: Option<String>,
    pub name: Option<String>,
    pub concept_type: Option<PayrollConceptType>,
    pub scope: Option<PayrollConceptScope>,
    pub period: Option<PayrollConceptPeriod>,
    pub active: Option<bool>,
}

/// Parameters for inserting a payroll concept into the repository.
#[derive(Debug, Clone)]
pub struct InsertPayrollConceptParams {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
    pub period: PayrollConceptPeriod,
    pub active: bool,
    pub payroll_id: Uuid,
}

#[async_trait]
pub trait PayrollConceptRepository: Send + Sync {
    async fn insert(&self, params: InsertPayrollConceptParams) -> AppResult<PayrollConcept>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollConcept>>;

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<PayrollConcept>>;

    async fn update(
        &self,
        id: Uuid,
        params: UpdatePayrollConceptParams,
    ) -> AppResult<Option<PayrollConcept>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PayrollConceptService {
    repository: Arc<dyn PayrollConceptRepository>,
    payroll_service: Arc<PayrollService>,
}

impl PayrollConceptService {
    pub fn new(
        repository: Arc<dyn PayrollConceptRepository>,
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
        params: CreatePayrollConceptParams,
    ) -> AppResult<PayrollConcept> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;

        let code = Self::normalize_code(&params.code)?;
        let name = Self::normalize_name(&params.name)?;

        self.repository
            .insert(InsertPayrollConceptParams {
                id: Uuid::new_v4(),
                code,
                name,
                concept_type: params.concept_type,
                scope: params.scope,
                period: params.period,
                active: params.active,
                payroll_id,
            })
            .await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        concept_id: Uuid,
    ) -> AppResult<Option<PayrollConcept>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;

        let concept = self.repository.fetch(concept_id).await?;
        Ok(concept.filter(|concept| concept.payroll_id == payroll_id))
    }

    pub async fn list(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<Vec<PayrollConcept>> {
        self.ensure_payroll_accessible(organization_id, payroll_id)
            .await?;

        let mut concepts = self.repository.fetch_by_payroll(payroll_id).await?;
        concepts.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(concepts)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        concept_id: Uuid,
        params: UpdatePayrollConceptParams,
    ) -> AppResult<Option<PayrollConcept>> {
        if params.code.is_none()
            && params.name.is_none()
            && params.concept_type.is_none()
            && params.scope.is_none()
            && params.period.is_none()
            && params.active.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self
            .get(organization_id, payroll_id, concept_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }

        let code = params
            .code
            .as_deref()
            .map(Self::normalize_code)
            .transpose()?;
        let name = params
            .name
            .as_deref()
            .map(Self::normalize_name)
            .transpose()?;

        self.repository
            .update(
                concept_id,
                UpdatePayrollConceptParams {
                    code,
                    name,
                    concept_type: params.concept_type,
                    scope: params.scope,
                    period: params.period,
                    active: params.active,
                },
            )
            .await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        concept_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, concept_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(concept_id).await
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

    fn normalize_code(value: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation("code cannot be empty"));
        }

        Ok(trimmed.to_string())
    }

    fn normalize_name(value: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation("name cannot be empty"));
        }

        Ok(trimmed.to_string())
    }
}
