use std::{io, sync::Arc};

use axum::Router;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::{
    infrastructure::{
        organization_repository::SurrealAnyOrganizationRepository,
        surreal::{self, SurrealConfig, SurrealConfigError},
    },
    routes,
    services::organization::OrganizationService,
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
}

impl AppState {
    pub fn new(organization_service: Arc<OrganizationService>) -> Self {
        Self {
            organization_service,
        }
    }

    pub fn organization_service(&self) -> Arc<OrganizationService> {
        Arc::clone(&self.organization_service)
    }

    pub async fn initialize() -> Result<Self, ServerSetupError> {
        let config = SurrealConfig::from_env()?;
        let client = surreal::connect(&config).await?;
        let repository: Arc<dyn crate::services::organization::OrganizationRepository> =
            Arc::new(SurrealAnyOrganizationRepository::new(client));
        let service = Arc::new(OrganizationService::new(repository));

        Ok(Self::new(service))
    }
}

#[derive(Debug, Error)]
pub enum ServerSetupError {
    #[error(transparent)]
    Config(#[from] SurrealConfigError),
    #[error(transparent)]
    Database(#[from] surrealdb::Error),
}
