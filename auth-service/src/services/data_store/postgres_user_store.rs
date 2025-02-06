use std::error::Error;

use crate::domain::{
    data_store::{UserStore, UserStoreError},
    Email, Password, User,
};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use argon2::password_hash::rand_core::OsRng;
use sqlx::PgPool;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let user_from_db = self.get_user(&user.get_email()).await;
        if user_from_db.is_ok() {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let password_hash = compute_password_hash(user.get_password().as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        sqlx::query!(r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.get_email().as_ref().to_string(),
            password_hash,
            user.use_requires_2fa()
          )
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let record = sqlx::query!(
           r#"
           SELECT * FROM users WHERE email = $1
           "#,
           email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        let email = Email::parse(record.email).map_err(|_| UserStoreError::UnexpectedError)?;
        let password = Password::parse(record.password_hash).map_err(|_| UserStoreError::UnexpectedError)?;

        let user = User::new(email, password, record.requires_2fa);
        Ok(user)
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        if user.get_email().as_ref() != email.as_ref() {
            return Err(UserStoreError::InvalidCredentials);
        }

        if verify_password_hash(user.get_password().as_ref().to_string(), password.as_ref().to_string()).await.is_err() {
            return Err(UserStoreError::InvalidCredentials);
        }

        Ok(())
    }
}


async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| Box::new(e))?;

        Ok(())
    }).await?
}

async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Box::new(e))?
            .to_string();

        Ok(password_hash)
    }).await?
}