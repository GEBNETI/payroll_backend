use std::sync::Arc;

use axum::Router;

use nomina::{
    routes,
    server::AppState,
    services::{
        division::{DivisionRepository, DivisionService},
        job::{JobRepository, JobService},
        organization::{OrganizationRepository, OrganizationService},
        payroll::{PayrollRepository, PayrollService},
    },
};

mod in_memory_repository;

pub use in_memory_repository::{
    InMemoryDivisionRepository, InMemoryJobRepository, InMemoryOrganizationRepository,
    InMemoryPayrollRepository,
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

    let state = AppState::new(
        organization_service,
        payroll_service,
        division_service,
        job_service,
    );

    routes::app_router(state)
}
