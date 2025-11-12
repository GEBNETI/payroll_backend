use axum::Json;

use crate::domain::health::HealthSnapshot;

pub async fn check() -> Json<HealthSnapshot> {
    Json(HealthSnapshot::current())
}
