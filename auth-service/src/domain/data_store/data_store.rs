use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::domain::{Email, Password, User};

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}


#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn storing_tokens(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn token_is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    BannedTokenNotFound,
    UnexpectedError
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(&mut self, email: Email, login_attempt_id: LoginAttemptId, code: TwoFACode) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        match uuid::Uuid::parse_str(&id) {
            Ok(uuid_id) => Ok(LoginAttemptId(uuid_id.to_string())),
            Err(_) => Err(format!("Invalid uuid: {}", id)),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = uuid::Uuid::new_v4().to_string().into();
        LoginAttemptId(uuid)
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        if code.len() != 6 {
            return Err(format!("Invalid code: {}", code));
        }

        Ok(TwoFACode(code))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let rnd_number = rng.random_range(100000..1000000);

        TwoFACode(rnd_number.to_string())
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}