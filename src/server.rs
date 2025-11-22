use std::{io, sync::Arc};

use axum::Router;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::{
    infrastructure::{
        bank_repository::SurrealAnyBankRepository,
        division_repository::SurrealAnyDivisionRepository,
        employee_repository::SurrealAnyEmployeeRepository,
        job_repository::SurrealAnyJobRepository,
        organization_repository::SurrealAnyOrganizationRepository,
        payroll_repository::SurrealAnyPayrollRepository,
        surreal::{self, SurrealConfig, SurrealConfigError},
    },
    routes,
    services::{
        bank::BankService,
        division::DivisionService,
        employee::EmployeeService,
        job::JobService,
        organization::{self, OrganizationService},
        payroll::PayrollService,
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
    bank_service: Arc<BankService>,
    employee_service: Arc<EmployeeService>,
}

impl AppState {
    pub fn new(
        organization_service: Arc<OrganizationService>,
        payroll_service: Arc<PayrollService>,
        division_service: Arc<DivisionService>,
        job_service: Arc<JobService>,
        bank_service: Arc<BankService>,
        employee_service: Arc<EmployeeService>,
    ) -> Self {
        Self {
            organization_service,
            payroll_service,
            division_service,
            job_service,
            bank_service,
            employee_service,
        }
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

    pub fn bank_service(&self) -> Arc<BankService> {
        Arc::clone(&self.bank_service)
    }

    pub fn employee_service(&self) -> Arc<EmployeeService> {
        Arc::clone(&self.employee_service)
    }

    pub async fn initialize() -> Result<Self, ServerSetupError> {
        let config = SurrealConfig::from_env()?;
        let client = surreal::connect(&config).await?;

        let organization_repository: Arc<dyn organization::OrganizationRepository> =
            Arc::new(SurrealAnyOrganizationRepository::new(client.clone()));
        let organization_service = Arc::new(OrganizationService::new(organization_repository));

        let payroll_repository: Arc<dyn crate::services::payroll::PayrollRepository> =
            Arc::new(SurrealAnyPayrollRepository::new(client.clone()));
        let payroll_service = Arc::new(PayrollService::new(
            payroll_repository,
            Arc::clone(&organization_service),
        ));

        let division_repository: Arc<dyn crate::services::division::DivisionRepository> =
            Arc::new(SurrealAnyDivisionRepository::new(client.clone()));
        let division_service = Arc::new(DivisionService::new(
            division_repository,
            Arc::clone(&payroll_service),
        ));

        let job_repository: Arc<dyn crate::services::job::JobRepository> =
            Arc::new(SurrealAnyJobRepository::new(client.clone()));
        let job_service = Arc::new(JobService::new(
            job_repository,
            Arc::clone(&payroll_service),
        ));

        let bank_repository: Arc<dyn crate::services::bank::BankRepository> =
            Arc::new(SurrealAnyBankRepository::new(client.clone()));
        let bank_service = Arc::new(BankService::new(
            bank_repository,
            Arc::clone(&organization_service),
        ));

        let employee_repository: Arc<dyn crate::services::employee::EmployeeRepository> =
            Arc::new(SurrealAnyEmployeeRepository::new(client));
        let employee_service = Arc::new(EmployeeService::new(
            employee_repository,
            Arc::clone(&division_service),
            Arc::clone(&payroll_service),
            Arc::clone(&job_service),
            Arc::clone(&bank_service),
        ));

        Ok(Self::new(
            organization_service,
            payroll_service,
            division_service,
            job_service,
            bank_service,
            employee_service,
        ))
    }
}

#[derive(Debug, Error)]
pub enum ServerSetupError {
    #[error(transparent)]
    Config(#[from] SurrealConfigError),
    #[error(transparent)]
    Database(#[from] surrealdb::Error),
}
