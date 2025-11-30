use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        payroll_concept::PayrollConceptScope, payroll_concept_definition::PayrollConceptDefinition,
    },
    error::{AppError, AppResult},
    services::payroll_concept::PayrollConceptService,
};

#[derive(Debug, Clone)]
pub struct CreatePayrollConceptDefinitionParams {
    pub formula: String,
    pub condition: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePayrollConceptDefinitionParams {
    pub formula: Option<String>,
    pub condition: Option<String>,
}

#[async_trait]
pub trait PayrollConceptDefinitionRepository: Send + Sync {
    async fn insert(
        &self,
        id: Uuid,
        payroll_concept_id: Uuid,
        formula: String,
        condition: String,
    ) -> AppResult<PayrollConceptDefinition>;

    async fn fetch_by_concept(
        &self,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<PayrollConceptDefinition>>;

    async fn update(
        &self,
        id: Uuid,
        formula: Option<String>,
        condition: Option<String>,
    ) -> AppResult<Option<PayrollConceptDefinition>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PayrollConceptDefinitionService {
    repository: Arc<dyn PayrollConceptDefinitionRepository>,
    payroll_concept_service: Arc<PayrollConceptService>,
}

impl PayrollConceptDefinitionService {
    pub fn new(
        repository: Arc<dyn PayrollConceptDefinitionRepository>,
        payroll_concept_service: Arc<PayrollConceptService>,
    ) -> Self {
        Self {
            repository,
            payroll_concept_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
        params: CreatePayrollConceptDefinitionParams,
    ) -> AppResult<PayrollConceptDefinition> {
        let concept = self
            .ensure_global_concept(organization_id, payroll_id, payroll_concept_id)
            .await?;

        if self
            .repository
            .fetch_by_concept(concept.id)
            .await?
            .is_some()
        {
            return Err(AppError::conflict(format!(
                "payroll concept `{}` already has a definition",
                concept.id
            )));
        }

        let formula = Self::normalize_field(&params.formula, "formula")?;
        let condition = Self::normalize_field(&params.condition, "condition")?;
        let id = Uuid::new_v4();

        self.repository
            .insert(id, concept.id, formula, condition)
            .await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        let concept = self
            .ensure_global_concept(organization_id, payroll_id, payroll_concept_id)
            .await?;

        self.repository.fetch_by_concept(concept.id).await
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
        params: UpdatePayrollConceptDefinitionParams,
    ) -> AppResult<Option<PayrollConceptDefinition>> {
        if params.formula.is_none() && params.condition.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let concept = self
            .ensure_global_concept(organization_id, payroll_id, payroll_concept_id)
            .await?;
        let existing = self.repository.fetch_by_concept(concept.id).await?;
        let Some(definition) = existing else {
            return Ok(None);
        };

        let formula = params
            .formula
            .as_deref()
            .map(|value| Self::normalize_field(value, "formula"))
            .transpose()?;
        let condition = params
            .condition
            .as_deref()
            .map(|value| Self::normalize_field(value, "condition"))
            .transpose()?;

        self.repository
            .update(definition.id, formula, condition)
            .await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<bool> {
        let concept = self
            .ensure_global_concept(organization_id, payroll_id, payroll_concept_id)
            .await?;

        let existing = self.repository.fetch_by_concept(concept.id).await?;
        let Some(definition) = existing else {
            return Ok(false);
        };

        self.repository.delete(definition.id).await
    }

    async fn ensure_global_concept(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<crate::domain::payroll_concept::PayrollConcept> {
        let concept = self
            .payroll_concept_service
            .get(organization_id, payroll_id, payroll_concept_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll concept `{payroll_concept_id}` not found for payroll `{payroll_id}`"
                ))
            })?;

        if concept.scope != PayrollConceptScope::Global {
            return Err(AppError::validation(
                "only global payroll concepts can define formulas",
            ));
        }

        Ok(concept)
    }

    fn normalize_field(value: &str, label: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(format!("{label} cannot be empty")));
        }

        Ok(trimmed.to_string())
    }
}
