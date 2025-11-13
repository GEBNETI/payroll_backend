use std::sync::Arc;

use axum::Router;

use nomina::{
    routes,
    server::AppState,
    services::organization::{OrganizationRepository, OrganizationService},
};

mod in_memory_repository;

pub use in_memory_repository::InMemoryOrganizationRepository;

pub fn test_router() -> Router {
    let repository: Arc<dyn OrganizationRepository> =
        Arc::new(InMemoryOrganizationRepository::default());
    let service = Arc::new(OrganizationService::new(repository));
    let state = AppState::new(service);

    routes::app_router(state)
}
