use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub is_active: bool,
}

impl User {
    pub fn new(
        id: Uuid,
        username: String,
        email: String,
        password_hash: String,
        name: String,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            name,
            is_active,
        }
    }
}
