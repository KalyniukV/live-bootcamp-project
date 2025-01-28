use std::string::ToString;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::{AuthAPIError, Email, Password}, utils::auth::generate_auth_cookie};
use crate::domain::{LoginAttemptId, TwoFACode};


pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) ->  Result<(CookieJar, impl IntoResponse), AuthAPIError> {

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }
    
    match user_store.get_user(&email).await {
        Ok(user) =>  {
            let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::IncorrectCredentials)?;
            let update_jar = jar.add(auth_cookie);
            
            match user.use_requires_2fa() {
                true => {
                    let response = handle_2fa(&email, &state).await?;
                    Ok((update_jar, response))
                },
                false => {
                    let response= handle_no_2fa().await?;
                    Ok((update_jar, response))
                }
            }
        }
        Err(_) => Err(AuthAPIError::UnexpectedError)
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState
) -> Result<(StatusCode, Json<LoginResponse>), AuthAPIError> {

    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let email_client = state.email_client.read().await;
    email_client.send_email(email, "2FA Code", two_fa_code.as_ref()).await.map_err(|_| AuthAPIError::UnexpectedError)?;

    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    two_fa_code_store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code).await.map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((StatusCode::PARTIAL_CONTENT,
       Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: login_attempt_id.as_ref().to_string()
       }))))
}

async fn handle_no_2fa() -> Result<(StatusCode, Json<LoginResponse>), AuthAPIError> {
    Ok((StatusCode::OK, Json(LoginResponse::RegularAuth)))
}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}