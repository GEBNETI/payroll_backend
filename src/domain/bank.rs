use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Bank metadata tied to a single organization.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct Bank {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
}

impl Bank {
    pub fn new(id: Uuid, name: impl Into<String>, organization_id: Uuid) -> Self {
        Self {
            id,
            name: name.into(),
            organization_id,
        }
    }
}
