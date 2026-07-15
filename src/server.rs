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
        role_repository::SurrealAnyRoleRepository,
        surreal::{self, SurrealConfig, SurrealConfigError},
        user_repository::SurrealAnyUserRepository,
        user_role_assignment_repository::SurrealAnyUserRoleAssignmentRepository,
    },
    routes,
    services::{
        auth::{AuthService, JwtConfig},
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
        payroll_history_report::PayrollHistoryReportService,
        role::{RoleRepository, RoleService},
        user::{InsertUserParams, UserRepository, UserService},
        user_role_assignment::{
            AssignRoleParams, UserRoleAssignmentRepository, UserRoleAssignmentService,
        },
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
    payroll_history_report_service: Arc<PayrollHistoryReportService>,
    payroll_calculator_service: Arc<PayrollCalculatorService>,
    user_service: Arc<UserService>,
    role_service: Arc<RoleService>,
    user_role_assignment_service: Arc<UserRoleAssignmentService>,
    auth_service: Arc<AuthService>,
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

    pub fn payroll_history_report_service(&self) -> Arc<PayrollHistoryReportService> {
        Arc::clone(&self.payroll_history_report_service)
    }

    pub fn payroll_calculator_service(&self) -> Arc<PayrollCalculatorService> {
        Arc::clone(&self.payroll_calculator_service)
    }

    pub fn user_service(&self) -> Arc<UserService> {
        Arc::clone(&self.user_service)
    }

    pub fn role_service(&self) -> Arc<RoleService> {
        Arc::clone(&self.role_service)
    }

    pub fn user_role_assignment_service(&self) -> Arc<UserRoleAssignmentService> {
        Arc::clone(&self.user_role_assignment_service)
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        Arc::clone(&self.auth_service)
    }

    pub async fn initialize() -> Result<Self, ServerSetupError> {
        let config = SurrealConfig::from_env()?;
        let client = surreal::connect(&config).await?;
        let jwt_config = JwtConfig::from_env().map_err(ServerSetupError::Init)?;
        let state = Self::builder()
            .with_surreal_repositories(client)
            .with_jwt_config(jwt_config)
            .build();
        state.seed_initial_data().await?;
        Ok(state)
    }

    async fn seed_initial_data(&self) -> Result<(), ServerSetupError> {
        use crate::domain::role::{
            ORGANIZATION_MANAGER_ROLE, PAYROLL_REPORT_ROLE, PAYROLL_USER_ROLE, SUPERUSER_ROLE,
        };

        let migrated_organizations = self
            .organization_service()
            .backfill_missing_rifs()
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?;
        if migrated_organizations > 0 {
            tracing::info!(
                migrated_organizations,
                "backfilled missing organization RIFs"
            );
        }

        self.role_service()
            .get_or_create(SUPERUSER_ROLE)
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?;
        self.role_service()
            .get_or_create(ORGANIZATION_MANAGER_ROLE)
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?;
        self.role_service()
            .get_or_create(PAYROLL_USER_ROLE)
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?;
        self.role_service()
            .get_or_create(PAYROLL_REPORT_ROLE)
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?;

        let root_username = std::env::var("ROOT_USERNAME").unwrap_or_else(|_| "root".to_string());

        if self
            .user_service()
            .get_by_username(&root_username)
            .await
            .map_err(|e| ServerSetupError::Seed(e.to_string()))?
            .is_none()
        {
            let root_password = std::env::var("ROOT_PASSWORD").map_err(|_| {
                ServerSetupError::Seed("ROOT_PASSWORD environment variable not set".to_string())
            })?;

            let password_hash = AuthService::hash_password(&root_password)
                .map_err(|e| ServerSetupError::Seed(e.to_string()))?;

            let root_user = self
                .user_service()
                .create(InsertUserParams {
                    username: root_username,
                    email: "root@localhost".to_string(),
                    password_hash,
                    name: "Root".to_string(),
                })
                .await
                .map_err(|e| ServerSetupError::Seed(e.to_string()))?;

            let superuser_role = self
                .role_service()
                .get_by_name(SUPERUSER_ROLE)
                .await
                .map_err(|e| ServerSetupError::Seed(e.to_string()))?
                .ok_or_else(|| {
                    ServerSetupError::Seed("Superuser role not found after creation".to_string())
                })?;

            self.user_role_assignment_service()
                .assign(AssignRoleParams {
                    user_id: root_user.id,
                    role_id: superuser_role.id,
                    role_name: superuser_role.name,
                    organization_id: None,
                    payroll_id: None,
                    payroll_name: None,
                })
                .await
                .map_err(|e| ServerSetupError::Seed(e.to_string()))?;

            tracing::info!("root user created successfully");
        }

        Ok(())
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
    user_repository: Option<Arc<dyn UserRepository>>,
    role_repository: Option<Arc<dyn RoleRepository>>,
    user_role_assignment_repository: Option<Arc<dyn UserRoleAssignmentRepository>>,
    jwt_config: Option<JwtConfig>,
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
            SurrealAnyPayrollHistoryDetailRepository::new(client.clone()),
        ));
        self.user_repository = Some(Arc::new(SurrealAnyUserRepository::new(client.clone())));
        self.role_repository = Some(Arc::new(SurrealAnyRoleRepository::new(client.clone())));
        self.user_role_assignment_repository = Some(Arc::new(
            SurrealAnyUserRoleAssignmentRepository::new(client),
        ));
        self
    }

    pub fn with_jwt_config(mut self, config: JwtConfig) -> Self {
        self.jwt_config = Some(config);
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

    pub fn with_user_repository(mut self, repo: Arc<dyn UserRepository>) -> Self {
        self.user_repository = Some(repo);
        self
    }

    pub fn with_role_repository(mut self, repo: Arc<dyn RoleRepository>) -> Self {
        self.role_repository = Some(repo);
        self
    }

    pub fn with_user_role_assignment_repository(
        mut self,
        repo: Arc<dyn UserRoleAssignmentRepository>,
    ) -> Self {
        self.user_role_assignment_repository = Some(repo);
        self
    }

    /// Build the `AppState`, wiring all services with their dependencies.
    ///
    /// # Panics
    ///
    /// Panics if any required repository or config is not set.
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
        let user_repository = self.user_repository.expect("user_repository is required");
        let role_repository = self.role_repository.expect("role_repository is required");
        let user_role_assignment_repository = self
            .user_role_assignment_repository
            .expect("user_role_assignment_repository is required");
        let jwt_config = self.jwt_config.expect("jwt_config is required");

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

        let payroll_history_report_service = Arc::new(PayrollHistoryReportService::new(
            Arc::clone(&payroll_history_service),
            Arc::clone(&payroll_history_detail_service),
            Arc::clone(&division_service),
            Arc::clone(&organization_service),
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

        let user_service = Arc::new(UserService::new(user_repository));
        let role_service = Arc::new(RoleService::new(role_repository));
        let user_role_assignment_service = Arc::new(UserRoleAssignmentService::new(
            user_role_assignment_repository,
        ));
        let auth_service = Arc::new(AuthService::new(jwt_config, Arc::clone(&user_service)));

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
            payroll_history_report_service,
            payroll_calculator_service,
            user_service,
            role_service,
            user_role_assignment_service,
            auth_service,
        }
    }
}

#[derive(Debug, Error)]
pub enum ServerSetupError {
    #[error(transparent)]
    Config(#[from] SurrealConfigError),
    #[error(transparent)]
    Database(#[from] surrealdb::Error),
    #[error("initialization error: {0}")]
    Init(String),
    #[error("seed error: {0}")]
    Seed(String),
}
