use std::sync::Arc;

use axum::Router;

use nomina::{
    routes,
    server::AppState,
    services::{
        organization::{OrganizationRepository, OrganizationService},
        payroll::{PayrollRepository, PayrollService},
    },
};

mod in_memory_repository;

pub use in_memory_repository::{InMemoryOrganizationRepository, InMemoryPayrollRepository};

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

    let state = AppState::new(organization_service, payroll_service);

    routes::app_router(state)
}
