use std::{io, sync::Arc};

use axum::Router;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::{
    infrastructure::{
        organization_repository::SurrealAnyOrganizationRepository,
        payroll_repository::SurrealAnyPayrollRepository,
        surreal::{self, SurrealConfig, SurrealConfigError},
    },
    routes,
    services::{
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
}

impl AppState {
    pub fn new(
        organization_service: Arc<OrganizationService>,
        payroll_service: Arc<PayrollService>,
    ) -> Self {
        Self {
            organization_service,
            payroll_service,
        }
    }

    pub fn organization_service(&self) -> Arc<OrganizationService> {
        Arc::clone(&self.organization_service)
    }

    pub fn payroll_service(&self) -> Arc<PayrollService> {
        Arc::clone(&self.payroll_service)
    }

    pub async fn initialize() -> Result<Self, ServerSetupError> {
        let config = SurrealConfig::from_env()?;
        let client = surreal::connect(&config).await?;

        let organization_repository: Arc<dyn organization::OrganizationRepository> =
            Arc::new(SurrealAnyOrganizationRepository::new(client.clone()));
        let organization_service = Arc::new(OrganizationService::new(organization_repository));

        let payroll_repository: Arc<dyn crate::services::payroll::PayrollRepository> =
            Arc::new(SurrealAnyPayrollRepository::new(client));
        let payroll_service = Arc::new(PayrollService::new(
            payroll_repository,
            Arc::clone(&organization_service),
        ));

        Ok(Self::new(organization_service, payroll_service))
    }
}

#[derive(Debug, Error)]
pub enum ServerSetupError {
    #[error(transparent)]
    Config(#[from] SurrealConfigError),
    #[error(transparent)]
    Database(#[from] surrealdb::Error),
}
