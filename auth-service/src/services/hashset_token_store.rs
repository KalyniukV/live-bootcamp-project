use std::collections::HashSet;

use crate::domain::BannedTokenStore;

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn storing_tokens(&mut self, token: String) {
        self.tokens.insert(token);
    }

    async fn token_is_banned(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn stored_token() {
        let mut banned_tokens_store = HashsetBannedTokenStore::default();

        let token  = "some_token".to_string();

        banned_tokens_store.storing_tokens(token.clone()).await;

        assert_eq!(banned_tokens_store.tokens.len(), 1);
        assert!(banned_tokens_store.tokens.contains(&token));
    }

    #[tokio::test]
    async fn banned_token() {
        let mut banned_tokens_store = HashsetBannedTokenStore::default();

        let token  = "some_token".to_string();

        banned_tokens_store.storing_tokens(token.clone()).await;

        let token_is_banned= banned_tokens_store.token_is_banned(&token).await;

        assert!(token_is_banned);
    }
}