use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::domain::{
    data_store::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let two_fa_instance = (login_attempt_id, code);

        let json_two_fa_instance = serde_json::to_string(&two_fa_instance)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex(key, json_two_fa_instance, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        self.conn
            .write()
            .await
            .del(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(&email);

        let two_fa_instance_value: String = self.conn
            .write()
            .await
            .get(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let result: (LoginAttemptId, TwoFACode) = serde_json::from_str(&two_fa_instance_value)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(result)
    }
}

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
