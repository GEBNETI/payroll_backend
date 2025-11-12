use axum::Router;

pub mod health;

pub fn app_router() -> Router {
    Router::new().merge(health::router())
}
