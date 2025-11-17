use axum::Json;

use crate::domain::health::Health;

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service health information", body = Health)),
    tag = "Health"
)]
pub async fn check() -> Json<Health> {
    Json(Health::current())
}
