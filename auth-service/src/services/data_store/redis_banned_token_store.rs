use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_store::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn storing_tokens(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let ttl = match TOKEN_TTL_SECONDS.try_into() {
            Ok(u_value) => u_value,
            Err(_) => return Err(BannedTokenStoreError::UnexpectedError)
        };

        let key = get_key(&token);

        self.conn
            .write()
            .await
            .set_ex(key, token, ttl)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn token_is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(&token);

        let result = self.conn
                .write()
                .await
                .exists(key)
                .map_err(|_| BannedTokenStoreError::BannedTokenNotFound)?;

        Ok(result)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
