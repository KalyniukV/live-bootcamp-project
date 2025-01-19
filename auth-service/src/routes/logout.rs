use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState, domain::AuthAPIError, 
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME}
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(c) => c,
        None => return (jar, Err(AuthAPIError::MissingToken))
    };

    let token = cookie.value().to_owned();
    
    if validate_token(&token, state.banned_token_store.clone()).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    let jar = jar.remove(JWT_COOKIE_NAME);

    let mut banned_token_store = state.banned_token_store.write().await;
    banned_token_store.storing_tokens(token.to_string()).await;

    (jar, Ok(StatusCode::OK))
}