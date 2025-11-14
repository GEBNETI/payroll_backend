use axum::Router;

use crate::server::AppState;

pub mod division;
pub mod health;
pub mod organization;
pub mod payroll;

pub fn app_router(state: AppState) -> Router {
    Router::<AppState>::new()
        .merge(health::router())
        .merge(organization::router())
        .merge(payroll::router())
        .merge(division::router())
        .with_state(state)
}
