use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, server::AppState};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/payrolls",
            post(handlers::payroll::create).get(handlers::payroll::list),
        )
        .route(
            "/payrolls/:id",
            get(handlers::payroll::get)
                .put(handlers::payroll::update)
                .delete(handlers::payroll::delete),
        )
}
