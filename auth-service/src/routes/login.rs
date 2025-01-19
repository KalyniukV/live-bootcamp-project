use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::auth::generate_auth_cookie,
};


pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) ->  (CookieJar, Result<impl IntoResponse, AuthAPIError>) {

    let email = match Email::parse(request.email) {
        Ok(e) => e,
        Err(_) => return (CookieJar::new(), Err(AuthAPIError::InvalidCredentials))
    };

    let password = match Password::parse(request.password) {
        Ok(p) => p,
        Err(_) => return (CookieJar::new(), Err(AuthAPIError::InvalidCredentials))
    };

    let user_store = state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return (CookieJar::new(),Err(AuthAPIError::IncorrectCredentials));
    }
    
    match user_store.get_user(&email).await {
        Ok(_) =>  {
            let auth_cookie=  match generate_auth_cookie(&email) {
                Ok(cookie) => cookie,
                Err(_) => return (CookieJar::new(), Err(AuthAPIError::UnexpectedError))
            };

            let update_jar = jar.add(auth_cookie);

            return (update_jar, Ok(StatusCode::OK.into_response()));
        }
        Err(_) => (CookieJar::new(), Err(AuthAPIError::UnexpectedError))
    }        
}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LoginResponse {
    pub message: String
}