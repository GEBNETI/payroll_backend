use axum::Json;

use crate::domain::health::HealthSnapshot;

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service health information", body = HealthSnapshot)),
    tag = "Health"
)]
pub async fn check() -> Json<HealthSnapshot> {
    Json(HealthSnapshot::current())
}
