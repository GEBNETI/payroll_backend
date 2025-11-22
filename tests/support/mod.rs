use std::sync::Arc;

use axum::Router;

use nomina::{
    routes,
    server::AppState,
    services::{
        bank::{BankRepository, BankService},
        division::{DivisionRepository, DivisionService},
        employee::{EmployeeRepository, EmployeeService},
        job::{JobRepository, JobService},
        organization::{OrganizationRepository, OrganizationService},
        payroll::{PayrollRepository, PayrollService},
    },
};

mod in_memory_repository;

pub use in_memory_repository::{
    InMemoryBankRepository, InMemoryDivisionRepository, InMemoryEmployeeRepository,
    InMemoryJobRepository, InMemoryOrganizationRepository, InMemoryPayrollRepository,
};

pub fn test_router() -> Router {
    let organization_repository: Arc<dyn OrganizationRepository> =
        Arc::new(InMemoryOrganizationRepository::default());
    let organization_service = Arc::new(OrganizationService::new(organization_repository));

    let payroll_repository: Arc<dyn PayrollRepository> =
        Arc::new(InMemoryPayrollRepository::default());
    let payroll_service = Arc::new(PayrollService::new(
        payroll_repository,
        Arc::clone(&organization_service),
    ));

    let division_repository: Arc<dyn DivisionRepository> =
        Arc::new(InMemoryDivisionRepository::default());
    let division_service = Arc::new(DivisionService::new(
        division_repository,
        Arc::clone(&payroll_service),
    ));

    let job_repository: Arc<dyn JobRepository> = Arc::new(InMemoryJobRepository::default());
    let job_service = Arc::new(JobService::new(
        job_repository,
        Arc::clone(&payroll_service),
    ));

    let bank_repository: Arc<dyn BankRepository> = Arc::new(InMemoryBankRepository::default());
    let bank_service = Arc::new(BankService::new(
        bank_repository,
        Arc::clone(&organization_service),
    ));

    let employee_repository: Arc<dyn EmployeeRepository> =
        Arc::new(InMemoryEmployeeRepository::default());
    let employee_service = Arc::new(EmployeeService::new(
        employee_repository,
        Arc::clone(&division_service),
        Arc::clone(&payroll_service),
        Arc::clone(&job_service),
        Arc::clone(&bank_service),
    ));

    let state = AppState::new(
        organization_service,
        payroll_service,
        division_service,
        job_service,
        bank_service,
        employee_service,
    );

    routes::app_router(state)
}
