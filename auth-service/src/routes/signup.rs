use axum::{
    extract::State, http::StatusCode, 
    response::{IntoResponse, Json}  
};
use serde::{Deserialize, Serialize};
use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User},
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {

    let email: Email;
    let password: Password;

    match Email::parse(request.email) {
        Ok(e) => {
            email = e;
        }
        Err(err) => {
            return Err(err);
        }
    }

    match Password::parse(request.password) {
        Ok(p) => {
            password = p;
        }
        Err(err) => {
            return Err(err);
        }
    }

    let mut user_store = state.user_store.write().await;
    if user_store.get_user(&email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let user = User::new(email, password, request.requires_2fa);
    
    let add_user_result = user_store.add_user(user).await;
    if add_user_result.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SignupResponse {
    pub message: String
}