use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        employee_payroll_concept::EmployeePayrollConcept,
        payroll_concept::{PayrollConcept, PayrollConceptScope},
    },
    error::{AppError, AppResult},
    services::{employee::EmployeeService, payroll_concept::PayrollConceptService},
};

#[derive(Debug, Clone)]
pub struct CreateEmployeePayrollConceptParams {
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateEmployeePayrollConceptParams {
    pub amount: Option<f64>,
}

#[async_trait]
pub trait EmployeePayrollConceptRepository: Send + Sync {
    async fn insert(
        &self,
        id: Uuid,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
        amount: f64,
    ) -> AppResult<EmployeePayrollConcept>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<EmployeePayrollConcept>>;

    async fn fetch_by_employee(&self, employee_id: Uuid) -> AppResult<Vec<EmployeePayrollConcept>>;

    async fn fetch_by_employee_and_concept(
        &self,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<Option<EmployeePayrollConcept>>;

    async fn update_amount(
        &self,
        id: Uuid,
        amount: f64,
    ) -> AppResult<Option<EmployeePayrollConcept>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct EmployeePayrollConceptService {
    repository: Arc<dyn EmployeePayrollConceptRepository>,
    employee_service: Arc<EmployeeService>,
    payroll_concept_service: Arc<PayrollConceptService>,
}

impl EmployeePayrollConceptService {
    pub fn new(
        repository: Arc<dyn EmployeePayrollConceptRepository>,
        employee_service: Arc<EmployeeService>,
        payroll_concept_service: Arc<PayrollConceptService>,
    ) -> Self {
        Self {
            repository,
            employee_service,
            payroll_concept_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
        params: CreateEmployeePayrollConceptParams,
    ) -> AppResult<EmployeePayrollConcept> {
        self.ensure_employee_accessible(organization_id, payroll_id, division_id, employee_id)
            .await?;
        let concept = self
            .ensure_individual_concept(organization_id, payroll_id, params.payroll_concept_id)
            .await?;
        self.ensure_unique_pair(employee_id, concept.id).await?;
        let amount = Self::validate_amount(params.amount)?;
        let id = Uuid::new_v4();

        self.repository
            .insert(id, employee_id, concept.id, amount)
            .await
    }

    pub async fn list(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
    ) -> AppResult<Vec<EmployeePayrollConcept>> {
        self.ensure_employee_accessible(organization_id, payroll_id, division_id, employee_id)
            .await?;

        let mut assignments = self.repository.fetch_by_employee(employee_id).await?;
        assignments.sort_by_key(|assignment| assignment.payroll_concept_id);
        Ok(assignments)
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        self.ensure_employee_accessible(organization_id, payroll_id, division_id, employee_id)
            .await?;

        let assignment = self.repository.fetch(assignment_id).await?;
        Ok(assignment.filter(|assignment| assignment.employee_id == employee_id))
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
        assignment_id: Uuid,
        params: UpdateEmployeePayrollConceptParams,
    ) -> AppResult<Option<EmployeePayrollConcept>> {
        if params.amount.is_none() {
            return Err(AppError::validation("no fields supplied for update"));
        }

        if self
            .get(
                organization_id,
                payroll_id,
                division_id,
                employee_id,
                assignment_id,
            )
            .await?
            .is_none()
        {
            return Ok(None);
        }

        let amount = params
            .amount
            .map(Self::validate_amount)
            .transpose()?
            .expect("validated amount expected");

        self.repository.update_amount(assignment_id, amount).await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(
                organization_id,
                payroll_id,
                division_id,
                employee_id,
                assignment_id,
            )
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(assignment_id).await
    }

    async fn ensure_employee_accessible(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
    ) -> AppResult<()> {
        let exists = self
            .employee_service
            .get(organization_id, payroll_id, division_id, employee_id)
            .await?
            .is_some();

        if exists {
            Ok(())
        } else {
            Err(AppError::not_found(format!(
                "employee `{employee_id}` not found for division `{division_id}` in payroll `{payroll_id}`"
            )))
        }
    }

    async fn ensure_individual_concept(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<PayrollConcept> {
        let concept = self
            .payroll_concept_service
            .get(organization_id, payroll_id, payroll_concept_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll concept `{payroll_concept_id}` not found for payroll `{payroll_id}`"
                ))
            })?;

        if concept.scope != PayrollConceptScope::Individual {
            return Err(AppError::validation(
                "only individual payroll concepts can be assigned to employees",
            ));
        }

        Ok(concept)
    }

    async fn ensure_unique_pair(
        &self,
        employee_id: Uuid,
        payroll_concept_id: Uuid,
    ) -> AppResult<()> {
        let existing = self
            .repository
            .fetch_by_employee_and_concept(employee_id, payroll_concept_id)
            .await?;

        if existing.is_some() {
            Err(AppError::conflict(format!(
                "payroll concept `{payroll_concept_id}` already assigned to employee `{employee_id}`"
            )))
        } else {
            Ok(())
        }
    }

    fn validate_amount(value: f64) -> AppResult<f64> {
        if value <= 0.0 {
            return Err(AppError::validation("amount must be greater than zero"));
        }

        Ok(value)
    }
}
