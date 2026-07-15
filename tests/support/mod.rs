use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, HeaderValue, Request},
    middleware::{self, Next},
    Router,
};
use uuid::Uuid;

use nomina::{domain::user::User, routes, server::AppState, services::auth::JwtConfig};

mod in_memory_repository;

pub use in_memory_repository::{
    InMemoryBankRepository, InMemoryDivisionRepository, InMemoryEmployeePayrollConceptRepository,
    InMemoryEmployeeRepository, InMemoryJobRepository, InMemoryOrganizationRepository,
    InMemoryPayrollConceptDefinitionRepository, InMemoryPayrollConceptRepository,
    InMemoryPayrollHistoryDetailRepository, InMemoryPayrollHistoryRepository,
    InMemoryPayrollRepository, InMemoryRoleRepository, InMemoryUserRepository,
    InMemoryUserRoleAssignmentRepository,
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
        .with_payroll_history_repository(Arc::new(InMemoryPayrollHistoryRepository::default()))
        .with_payroll_history_detail_repository(Arc::new(
            InMemoryPayrollHistoryDetailRepository::default(),
        ))
        .with_user_repository(Arc::new(InMemoryUserRepository::default()))
        .with_role_repository(Arc::new(InMemoryRoleRepository::default()))
        .with_user_role_assignment_repository(Arc::new(
            InMemoryUserRoleAssignmentRepository::default(),
        ))
        .with_jwt_config(JwtConfig {
            secret: "test-secret-that-is-at-least-32-characters".to_string(),
            access_expiry_minutes: 15,
            refresh_expiry_days: 7,
        })
        .build();

    let test_user = User::new(
        Uuid::new_v4(),
        "test-superuser".to_string(),
        "test@example.com".to_string(),
        "unused-password-hash".to_string(),
        "Test Superuser".to_string(),
        true,
    );
    let token = state
        .auth_service()
        .generate_access_token(&test_user, true)
        .expect("test access token");
    let authorization =
        HeaderValue::from_str(&format!("Bearer {token}")).expect("test authorization header");

    routes::app_router(state).layer(middleware::from_fn(
        move |mut request: Request<Body>, next: Next| {
            let authorization = authorization.clone();
            async move {
                request
                    .headers_mut()
                    .insert(header::AUTHORIZATION, authorization);
                next.run(request).await
            }
        },
    ))
}
