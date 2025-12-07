use std::sync::Arc;

use axum::Router;

use nomina::{routes, server::AppState};

mod in_memory_repository;

pub use in_memory_repository::{
    InMemoryBankRepository, InMemoryDivisionRepository, InMemoryEmployeePayrollConceptRepository,
    InMemoryEmployeeRepository, InMemoryJobRepository, InMemoryOrganizationRepository,
    InMemoryPayrollConceptDefinitionRepository, InMemoryPayrollConceptRepository,
    InMemoryPayrollRepository,
};

pub fn test_router() -> Router {
    let state = AppState::builder()
        .with_organization_repository(Arc::new(InMemoryOrganizationRepository::default()))
        .with_payroll_repository(Arc::new(InMemoryPayrollRepository::default()))
        .with_division_repository(Arc::new(InMemoryDivisionRepository::default()))
        .with_job_repository(Arc::new(InMemoryJobRepository::default()))
        .with_bank_repository(Arc::new(InMemoryBankRepository::default()))
        .with_employee_repository(Arc::new(InMemoryEmployeeRepository::default()))
        .with_payroll_concept_repository(Arc::new(InMemoryPayrollConceptRepository::default()))
        .with_payroll_concept_definition_repository(Arc::new(
            InMemoryPayrollConceptDefinitionRepository::default(),
        ))
        .with_employee_payroll_concept_repository(Arc::new(
            InMemoryEmployeePayrollConceptRepository::default(),
        ))
        .build();

    routes::app_router(state)
}
