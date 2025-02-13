use auth_service::{app_state::AppState, get_postgres_pool, get_redis_client, services::data_store::{postgres_user_store::PostgresUserStore,
                                                                                                    hashset_token_store::HashsetBannedTokenStore,
                                                                                                    hashmap_two_fa_code_store::HashmapTwoFACodeStore}, utils::prod, Application};
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use auth_service::domain::MockEmailClient;
use auth_service::services::data_store::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::data_store::redis_two_fa_code_store::RedisTwoFACodeStore;
use auth_service::utils::{constants, REDIS_HOST_NAME};
use constants::{DATABASE_URL};


#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(RwLock::new(configure_redis()));

    let user_store  = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store  = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection.clone())));
    let two_fa_code_store  = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection.clone())));
    let email_client = Arc::new(RwLock::new(MockEmailClient));

    let app_state = AppState::new(user_store, banned_token_store, two_fa_code_store, email_client);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}