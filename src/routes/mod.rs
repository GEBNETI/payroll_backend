use axum::Router;

use crate::server::AppState;

pub mod health;
pub mod organization;

pub fn app_router(state: AppState) -> Router {
    Router::<AppState>::new()
        .merge(health::router())
        .merge(organization::router())
        .with_state(state)
}
