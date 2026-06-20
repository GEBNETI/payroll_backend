use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::user::User,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct InsertUserParams {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateUserParams {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, id: Uuid, params: InsertUserParams) -> AppResult<User>;
    async fn fetch(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn fetch_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn fetch_all(&self) -> AppResult<Vec<User>>;
    async fn update(&self, id: Uuid, params: UpdateUserParams) -> AppResult<Option<User>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, params: InsertUserParams) -> AppResult<User> {
        let id = Uuid::new_v4();
        self.repository.insert(id, params).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<User>> {
        self.repository.fetch(id).await
    }

    pub async fn get_by_username(&self, username: &str) -> AppResult<Option<User>> {
        self.repository.fetch_by_username(username).await
    }

    pub async fn list(&self) -> AppResult<Vec<User>> {
        let mut users = self.repository.fetch_all().await?;
        users.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(users)
    }

    pub async fn update(&self, id: Uuid, params: UpdateUserParams) -> AppResult<Option<User>> {
        if params.email.is_none()
            && params.name.is_none()
            && params.password_hash.is_none()
            && params.is_active.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }
        self.repository.update(id, params).await
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}
