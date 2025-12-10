use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::payroll_history_detail::{NewPayrollHistoryDetailData, PayrollHistoryDetail},
    error::{AppError, AppResult},
    services::{
        bank::BankService, division::DivisionService, employee::EmployeeService, job::JobService,
        payroll_concept::PayrollConceptService, payroll_history::PayrollHistoryService,
    },
};

#[derive(Debug, Clone)]
pub struct CreatePayrollHistoryDetailParams {
    pub employee_id: Uuid,
    pub payroll_concept_id: Uuid,
    pub amount: f64,
}

#[async_trait]
pub trait PayrollHistoryDetailRepository: Send + Sync {
    async fn insert(&self, data: NewPayrollHistoryDetailData) -> AppResult<PayrollHistoryDetail>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollHistoryDetail>>;

    async fn fetch_by_payroll_history(
        &self,
        payroll_history_id: Uuid,
    ) -> AppResult<Vec<PayrollHistoryDetail>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PayrollHistoryDetailService {
    repository: Arc<dyn PayrollHistoryDetailRepository>,
    payroll_history_service: Arc<PayrollHistoryService>,
    employee_service: Arc<EmployeeService>,
    division_service: Arc<DivisionService>,
    job_service: Arc<JobService>,
    bank_service: Arc<BankService>,
    payroll_concept_service: Arc<PayrollConceptService>,
}

impl PayrollHistoryDetailService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repository: Arc<dyn PayrollHistoryDetailRepository>,
        payroll_history_service: Arc<PayrollHistoryService>,
        employee_service: Arc<EmployeeService>,
        division_service: Arc<DivisionService>,
        job_service: Arc<JobService>,
        bank_service: Arc<BankService>,
        payroll_concept_service: Arc<PayrollConceptService>,
    ) -> Self {
        Self {
            repository,
            payroll_history_service,
            employee_service,
            division_service,
            job_service,
            bank_service,
            payroll_concept_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        params: CreatePayrollHistoryDetailParams,
    ) -> AppResult<PayrollHistoryDetail> {
        // Validate payroll history exists
        let history = self
            .payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;

        // Validate amount
        Self::validate_amount(params.amount)?;

        // Fetch employee by searching across all divisions
        let employee = self
            .find_employee_by_id(organization_id, payroll_id, params.employee_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "employee `{}` not found for payroll `{}`",
                    params.employee_id, payroll_id
                ))
            })?;

        // Fetch division
        let division = self
            .division_service
            .get(organization_id, payroll_id, employee.division_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("division `{}` not found", employee.division_id))
            })?;

        // Fetch job
        let job = self
            .job_service
            .get(organization_id, payroll_id, employee.job_id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("job `{}` not found", employee.job_id)))?;

        // Fetch bank
        let bank = self
            .bank_service
            .get(organization_id, employee.bank_id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("bank `{}` not found", employee.bank_id)))?;

        // Fetch payroll concept
        let concept = self
            .payroll_concept_service
            .get(organization_id, payroll_id, params.payroll_concept_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll concept `{}` not found",
                    params.payroll_concept_id
                ))
            })?;

        let id = Uuid::new_v4();

        let data = NewPayrollHistoryDetailData {
            id,
            payroll_history_id: history.id,
            division_id: division.id,
            division_name: division.name,
            division_budget_code: division.budget_code,
            job_id: job.id,
            job_title: job.job_title,
            job_salary: job.salary,
            employee_id: employee.id,
            employee_id_number: employee.id_number,
            employee_last_name: employee.last_name,
            employee_first_name: employee.first_name,
            employee_hire_date: employee.hire_date,
            employee_salary: employee.salary,
            employee_clasification: employee.clasification,
            employee_hours: employee.hours,
            employee_bank_account: employee.bank_account,
            bank_id: bank.id,
            bank_name: bank.name,
            payroll_concept_id: concept.id,
            payroll_concept_code: concept.code,
            payroll_concept_name: concept.name,
            payroll_concept_type: concept.concept_type,
            payroll_concept_scope: concept.scope,
            payroll_concept_period: concept.period,
            amount: params.amount,
        };

        self.repository.insert(data).await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        detail_id: Uuid,
    ) -> AppResult<Option<PayrollHistoryDetail>> {
        // Validate payroll history exists
        self.payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;

        let detail = self.repository.fetch(detail_id).await?;
        Ok(detail.filter(|d| d.payroll_history_id == history_id))
    }

    pub async fn list(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<Vec<PayrollHistoryDetail>> {
        // Validate payroll history exists
        self.payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;

        let mut details = self.repository.fetch_by_payroll_history(history_id).await?;
        details.sort_by(|a, b| {
            a.employee_last_name
                .cmp(&b.employee_last_name)
                .then_with(|| a.employee_first_name.cmp(&b.employee_first_name))
                .then_with(|| a.payroll_concept_code.cmp(&b.payroll_concept_code))
        });
        Ok(details)
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        detail_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, history_id, detail_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(detail_id).await
    }

    /// Find an employee by ID and verify they belong to the payroll
    async fn find_employee_by_id(
        &self,
        _organization_id: Uuid,
        payroll_id: Uuid,
        employee_id: Uuid,
    ) -> AppResult<Option<crate::domain::employee::Employee>> {
        let employee = self.employee_service.get_by_id(employee_id).await?;
        Ok(employee.filter(|e| e.payroll_id == payroll_id))
    }

    fn validate_amount(value: f64) -> AppResult<()> {
        if !value.is_finite() {
            return Err(AppError::validation("amount must be a valid number"));
        }
        if value < 0.0 {
            return Err(AppError::validation("amount cannot be negative"));
        }
        Ok(())
    }
}
