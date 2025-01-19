use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, UserStore};

pub type UserStoreType = Arc<RwLock<dyn UserStore>>;
pub type BannedTokenSoreType = Arc<RwLock<dyn BannedTokenStore>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenSoreType
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedTokenSoreType) -> Self {
        Self { user_store, banned_token_store }
    }
}