use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    pub application: &'static str,
    pub authors: &'static str,
    pub version: &'static str,
}

impl Health {
    pub fn current() -> Self {
        Self {
            application: env!("CARGO_PKG_NAME"),
            authors: env!("CARGO_PKG_AUTHORS"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
