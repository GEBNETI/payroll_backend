use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    domain::user::User,
    error::{AppError, AppResult},
    services::user::UserService,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    pub sub: String,
    pub username: String,
    pub name: String,
    pub is_superuser: bool,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry_minutes: u64,
    pub refresh_expiry_days: u64,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, String> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET environment variable not set".to_string())?;
        if secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".to_string());
        }
        let access_expiry_minutes = std::env::var("JWT_ACCESS_EXPIRY_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .unwrap_or(15u64);
        let refresh_expiry_days = std::env::var("JWT_REFRESH_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .unwrap_or(7u64);
        Ok(Self {
            secret,
            access_expiry_minutes,
            refresh_expiry_days,
        })
    }
}

#[derive(Clone)]
pub struct AuthService {
    jwt_config: JwtConfig,
    user_service: Arc<UserService>,
}

impl AuthService {
    pub fn new(jwt_config: JwtConfig, user_service: Arc<UserService>) -> Self {
        Self {
            jwt_config,
            user_service,
        }
    }

    pub fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::internal(format!("password hashing failed: {e}")))
    }

    pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::internal(format!("invalid password hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> AppResult<User> {
        let user = self
            .user_service
            .get_by_username(username)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid credentials"))?;

        if !user.is_active {
            return Err(AppError::unauthorized("account is disabled"));
        }

        if !Self::verify_password(password, &user.password_hash)? {
            return Err(AppError::unauthorized("invalid credentials"));
        }

        Ok(user)
    }

    pub fn generate_access_token(&self, user: &User, is_superuser: bool) -> AppResult<String> {
        let now = now_secs();
        let claims = AccessClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            name: user.name.clone(),
            is_superuser,
            iat: now,
            exp: now + self.jwt_config.access_expiry_minutes * 60,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|e| AppError::internal(format!("failed to generate access token: {e}")))
    }

    pub fn generate_refresh_token(&self, user: &User) -> AppResult<String> {
        let now = now_secs();
        let claims = RefreshClaims {
            sub: user.id.to_string(),
            iat: now,
            exp: now + self.jwt_config.refresh_expiry_days * 24 * 3600,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )
        .map_err(|e| AppError::internal(format!("failed to generate refresh token: {e}")))
    }

    pub fn validate_access_token(&self, token: &str) -> AppResult<AccessClaims> {
        decode::<AccessClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::unauthorized("invalid or expired access token"))
    }

    pub fn validate_refresh_token(&self, token: &str) -> AppResult<RefreshClaims> {
        decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::unauthorized("invalid or expired refresh token"))
    }

    pub fn refresh_expiry_seconds(&self) -> u64 {
        self.jwt_config.refresh_expiry_days * 24 * 3600
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
