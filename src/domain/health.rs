use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthSnapshot {
    pub application: &'static str,
    pub authors: &'static str,
    pub version: &'static str,
}

impl HealthSnapshot {
    pub fn current() -> Self {
        Self {
            application: env!("CARGO_PKG_NAME"),
            authors: env!("CARGO_PKG_AUTHORS"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
