use std::env;

use surrealdb::{
    Surreal,
    engine::any::{self, Any},
    opt::auth::Root,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SurrealConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl SurrealConfig {
    pub fn from_env() -> Result<Self, SurrealConfigError> {
        Ok(Self {
            url: read_env("SURREALDB_URL")?,
            namespace: read_env("SURREALDB_NAMESPACE")?,
            database: read_env("SURREALDB_DATABASE")?,
            username: read_env("SURREALDB_USERNAME")?,
            password: read_env("SURREALDB_PASSWORD")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum SurrealConfigError {
    #[error("missing `{0}` environment variable")]
    MissingEnv(&'static str),
}

fn read_env(key: &'static str) -> Result<String, SurrealConfigError> {
    env::var(key).map_err(|_| SurrealConfigError::MissingEnv(key))
}

pub async fn connect(config: &SurrealConfig) -> Result<Surreal<Any>, surrealdb::Error> {
    let client = any::connect(&config.url).await?;

    client
        .signin(Root {
            username: &config.username,
            password: &config.password,
        })
        .await?;

    client
        .use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;

    Ok(client)
}
