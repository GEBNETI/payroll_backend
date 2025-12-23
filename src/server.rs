use std::{io, sync::Arc};

use axum::Router;
use surrealdb::{engine::any::Any, Surreal};
use thiserror::Error;
use tokio::net::TcpListener;

use crate::{
    infrastructure::{
        bank_repository::SurrealAnyBankRepository,
        division_repository::SurrealAnyDivisionRepository,
        employee_payroll_concept_repository::SurrealAnyEmployeePayrollConceptRepository,
        employee_repository::SurrealAnyEmployeeRepository,
        job_repository::SurrealAnyJobRepository,
        organization_repository::SurrealAnyOrganizationRepository,
        payroll_concept_definition_repository::SurrealAnyPayrollConceptDefinitionRepository,
        payroll_concept_repository::SurrealAnyPayrollConceptRepository,
        payroll_history_detail_repository::SurrealAnyPayrollHistoryDetailRepository,
        payroll_history_repository::SurrealAnyPayrollHistoryRepository,
        payroll_repository::SurrealAnyPayrollRepository,
        surreal::{self, SurrealConfig, SurrealConfigError},
    },
    routes,
    services::{
        bank::{BankRepository, BankService},
        division::{DivisionRepository, DivisionService},
        employee::{EmployeeRepository, EmployeeService},
        employee_payroll_concept::{
            EmployeePayrollConceptRepository, EmployeePayrollConceptService,
        },
        job::{JobRepository, JobService},
        organization::{OrganizationRepository, OrganizationService},
        payroll::{PayrollRepository, PayrollService},
        payroll_calculator::PayrollCalculatorService,
        payroll_concept::{PayrollConceptRepository, PayrollConceptService},
        payroll_concept_definition::{
            PayrollConceptDefinitionRepository, PayrollConceptDefinitionService,
        },
        payroll_history::{PayrollHistoryRepository, PayrollHistoryService},
        payroll_history_detail::{PayrollHistoryDetailRepository, PayrollHistoryDetailService},
    },
};

pub async fn run(listener: TcpListener) -> Result<(), io::Error> {
    let state = AppState::initialize()
        .await
        .map_err(|err| io::Error::other(err.to_string()))?;

    let app = router(state);
    axum::serve(listener, app).await
}

pub fn router(state: AppState) -> Router {
    routes::app_router(state)
}

#[derive(Clone)]
pub struct AppState {
    organization_service: Arc<OrganizationService>,
    payroll_service: Arc<PayrollService>,
    division_service: Arc<DivisionService>,
    job_service: Arc<JobService>,
    employee_payroll_concept_service: Arc<EmployeePayrollConceptService>,
    payroll_concept_service: Arc<PayrollConceptService>,
    payroll_concept_definition_service: Arc<PayrollConceptDefinitionService>,
    bank_service: Arc<BankService>,
    employee_service: Arc<EmployeeService>,
    payroll_history_service: Arc<PayrollHistoryService>,
    payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
    payroll_calculator_service: Arc<PayrollCalculatorService>,
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }

    pub fn organization_service(&self) -> Arc<OrganizationService> {
        Arc::clone(&self.organization_service)
    }

    pub fn payroll_service(&self) -> Arc<PayrollService> {
        Arc::clone(&self.payroll_service)
    }

    pub fn division_service(&self) -> Arc<DivisionService> {
        Arc::clone(&self.division_service)
    }

    pub fn job_service(&self) -> Arc<JobService> {
        Arc::clone(&self.job_service)
    }

    pub fn employee_payroll_concept_service(&self) -> Arc<EmployeePayrollConceptService> {
        Arc::clone(&self.employee_payroll_concept_service)
    }

    pub fn payroll_concept_service(&self) -> Arc<PayrollConceptService> {
        Arc::clone(&self.payroll_concept_service)
    }

    pub fn payroll_concept_definition_service(&self) -> Arc<PayrollConceptDefinitionService> {
        Arc::clone(&self.payroll_concept_definition_service)
    }

    pub fn bank_service(&self) -> Arc<BankService> {
        Arc::clone(&self.bank_service)
    }

    pub fn employee_service(&self) -> Arc<EmployeeService> {
        Arc::clone(&self.employee_service)
    }

    pub fn payroll_history_service(&self) -> Arc<PayrollHistoryService> {
        Arc::clone(&self.payroll_history_service)
    }

    pub fn payroll_history_detail_service(&self) -> Arc<PayrollHistoryDetailService> {
        Arc::clone(&self.payroll_history_detail_service)
    }

    pub fn payroll_calculator_service(&self) -> Arc<PayrollCalculatorService> {
        Arc::clone(&self.payroll_calculator_service)
    }

    pub async fn initialize() -> Result<Self, ServerSetupError> {
        let config = SurrealConfig::from_env()?;
        let client = surreal::connect(&config).await?;
        Ok(Self::builder().with_surreal_repositories(client).build())
    }
}

/// Builder for `AppState` that wires services with their dependencies.
///
/// Use `with_surreal_repositories` for production or provide custom repositories
/// for testing via the individual `with_*_repository` methods.
#[derive(Default)]
pub struct AppStateBuilder {
    organization_repository: Option<Arc<dyn OrganizationRepository>>,
    payroll_repository: Option<Arc<dyn PayrollRepository>>,
    division_repository: Option<Arc<dyn DivisionRepository>>,
    job_repository: Option<Arc<dyn JobRepository>>,
    bank_repository: Option<Arc<dyn BankRepository>>,
    employee_repository: Option<Arc<dyn EmployeeRepository>>,
    payroll_concept_repository: Option<Arc<dyn PayrollConceptRepository>>,
    payroll_concept_definition_repository: Option<Arc<dyn PayrollConceptDefinitionRepository>>,
    employee_payroll_concept_repository: Option<Arc<dyn EmployeePayrollConceptRepository>>,
    payroll_history_repository: Option<Arc<dyn PayrollHistoryRepository>>,
    payroll_history_detail_repository: Option<Arc<dyn PayrollHistoryDetailRepository>>,
}

impl AppStateBuilder {
    /// Configure all repositories to use SurrealDB with the given client.
    pub fn with_surreal_repositories(mut self, client: Surreal<Any>) -> Self {
        self.organization_repository = Some(Arc::new(SurrealAnyOrganizationRepository::new(
            client.clone(),
        )));
        self.payroll_repository = Some(Arc::new(SurrealAnyPayrollRepository::new(client.clone())));
        self.division_repository =
            Some(Arc::new(SurrealAnyDivisionRepository::new(client.clone())));
        self.job_repository = Some(Arc::new(SurrealAnyJobRepository::new(client.clone())));
        self.bank_repository = Some(Arc::new(SurrealAnyBankRepository::new(client.clone())));
        self.employee_repository =
            Some(Arc::new(SurrealAnyEmployeeRepository::new(client.clone())));
        self.payroll_concept_repository = Some(Arc::new(SurrealAnyPayrollConceptRepository::new(
            client.clone(),
        )));
        self.payroll_concept_definition_repository = Some(Arc::new(
            SurrealAnyPayrollConceptDefinitionRepository::new(client.clone()),
        ));
        self.employee_payroll_concept_repository = Some(Arc::new(
            SurrealAnyEmployeePayrollConceptRepository::new(client.clone()),
        ));
        self.payroll_history_repository = Some(Arc::new(SurrealAnyPayrollHistoryRepository::new(
            client.clone(),
        )));
        self.payroll_history_detail_repository = Some(Arc::new(
            SurrealAnyPayrollHistoryDetailRepository::new(client),
        ));
        self
    }

    pub fn with_organization_repository(mut self, repo: Arc<dyn OrganizationRepository>) -> Self {
        self.organization_repository = Some(repo);
        self
    }

    pub fn with_payroll_repository(mut self, repo: Arc<dyn PayrollRepository>) -> Self {
        self.payroll_repository = Some(repo);
        self
    }

    pub fn with_division_repository(mut self, repo: Arc<dyn DivisionRepository>) -> Self {
        self.division_repository = Some(repo);
        self
    }

    pub fn with_job_repository(mut self, repo: Arc<dyn JobRepository>) -> Self {
        self.job_repository = Some(repo);
        self
    }

    pub fn with_bank_repository(mut self, repo: Arc<dyn BankRepository>) -> Self {
        self.bank_repository = Some(repo);
        self
    }

    pub fn with_employee_repository(mut self, repo: Arc<dyn EmployeeRepository>) -> Self {
        self.employee_repository = Some(repo);
        self
    }

    pub fn with_payroll_concept_repository(
        mut self,
        repo: Arc<dyn PayrollConceptRepository>,
    ) -> Self {
        self.payroll_concept_repository = Some(repo);
        self
    }

    pub fn with_payroll_concept_definition_repository(
        mut self,
        repo: Arc<dyn PayrollConceptDefinitionRepository>,
    ) -> Self {
        self.payroll_concept_definition_repository = Some(repo);
        self
    }

    pub fn with_employee_payroll_concept_repository(
        mut self,
        repo: Arc<dyn EmployeePayrollConceptRepository>,
    ) -> Self {
        self.employee_payroll_concept_repository = Some(repo);
        self
    }

    pub fn with_payroll_history_repository(
        mut self,
        repo: Arc<dyn PayrollHistoryRepository>,
    ) -> Self {
        self.payroll_history_repository = Some(repo);
        self
    }

    pub fn with_payroll_history_detail_repository(
        mut self,
        repo: Arc<dyn PayrollHistoryDetailRepository>,
    ) -> Self {
        self.payroll_history_detail_repository = Some(repo);
        self
    }

    /// Build the `AppState`, wiring all services with their dependencies.
    ///
    /// # Panics
    ///
    /// Panics if any repository is not set.
    pub fn build(self) -> AppState {
        // Extract repositories (panic if missing)
        let organization_repository = self
            .organization_repository
            .expect("organization_repository is required");
        let payroll_repository = self
            .payroll_repository
            .expect("payroll_repository is required");
        let division_repository = self
            .division_repository
            .expect("division_repository is required");
        let job_repository = self.job_repository.expect("job_repository is required");
        let bank_repository = self.bank_repository.expect("bank_repository is required");
        let employee_repository = self
            .employee_repository
            .expect("employee_repository is required");
        let payroll_concept_repository = self
            .payroll_concept_repository
            .expect("payroll_concept_repository is required");
        let payroll_concept_definition_repository = self
            .payroll_concept_definition_repository
            .expect("payroll_concept_definition_repository is required");
        let employee_payroll_concept_repository = self
            .employee_payroll_concept_repository
            .expect("employee_payroll_concept_repository is required");
        let payroll_history_repository = self
            .payroll_history_repository
            .expect("payroll_history_repository is required");
        let payroll_history_detail_repository = self
            .payroll_history_detail_repository
            .expect("payroll_history_detail_repository is required");

        // Build services in dependency order
        let organization_service = Arc::new(OrganizationService::new(organization_repository));

        let payroll_service = Arc::new(PayrollService::new(
            payroll_repository,
            Arc::clone(&organization_service),
        ));

        let division_service = Arc::new(DivisionService::new(
            division_repository,
            Arc::clone(&payroll_service),
        ));

        let job_service = Arc::new(JobService::new(
            job_repository,
            Arc::clone(&payroll_service),
        ));

        let bank_service = Arc::new(BankService::new(
            bank_repository,
            Arc::clone(&organization_service),
        ));

        let employee_service = Arc::new(EmployeeService::new(
            employee_repository,
            Arc::clone(&division_service),
            Arc::clone(&payroll_service),
            Arc::clone(&job_service),
            Arc::clone(&bank_service),
        ));

        let payroll_concept_service = Arc::new(PayrollConceptService::new(
            payroll_concept_repository,
            Arc::clone(&payroll_service),
        ));

        let payroll_concept_definition_service = Arc::new(PayrollConceptDefinitionService::new(
            payroll_concept_definition_repository,
            Arc::clone(&payroll_concept_service),
        ));

        let employee_payroll_concept_service = Arc::new(EmployeePayrollConceptService::new(
            employee_payroll_concept_repository,
            Arc::clone(&employee_service),
            Arc::clone(&payroll_concept_service),
        ));

        let payroll_history_service = Arc::new(PayrollHistoryService::new(
            payroll_history_repository,
            Arc::clone(&payroll_service),
            Arc::clone(&organization_service),
        ));

        let payroll_history_detail_service = Arc::new(PayrollHistoryDetailService::new(
            payroll_history_detail_repository,
            Arc::clone(&payroll_history_service),
            Arc::clone(&employee_service),
            Arc::clone(&division_service),
            Arc::clone(&job_service),
            Arc::clone(&bank_service),
            Arc::clone(&payroll_concept_service),
        ));

        let payroll_calculator_service = Arc::new(PayrollCalculatorService::new(
            Arc::clone(&payroll_history_service),
            Arc::clone(&payroll_history_detail_service),
            Arc::clone(&division_service),
            Arc::clone(&employee_service),
            Arc::clone(&employee_payroll_concept_service),
            Arc::clone(&payroll_concept_service),
            Arc::clone(&payroll_concept_definition_service),
            Arc::clone(&job_service),
        ));

        AppState {
            organization_service,
            payroll_service,
            division_service,
            job_service,
            employee_payroll_concept_service,
            payroll_concept_service,
            payroll_concept_definition_service,
            bank_service,
            employee_service,
            payroll_history_service,
            payroll_history_detail_service,
            payroll_calculator_service,
        }
    }
}

#[derive(Debug, Error)]
pub enum ServerSetupError {
    #[error(transparent)]
    Config(#[from] SurrealConfigError),
    #[error(transparent)]
    Database(#[from] surrealdb::Error),
}
