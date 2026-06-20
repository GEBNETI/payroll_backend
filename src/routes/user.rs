use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{handlers::user, server::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(user::create).get(user::list))
        .route(
            "/users/{id}",
            get(user::get).put(user::update).delete(user::delete),
        )
        .route(
            "/users/{id}/assignments",
            get(user::list_assignments).post(user::assign_role),
        )
        .route(
            "/users/{id}/assignments/{assignment_id}",
            delete(user::revoke_role),
        )
}
