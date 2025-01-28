use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode};
use crate::utils::auth::generate_auth_cookie;

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let request_login_attempt_id = LoginAttemptId::parse(request.loginAttemptId).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let request_two_fa_code = TwoFACode::parse(request.two_fa_code).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    let (state_login_attempt_id, state_two_fa_code) = two_fa_code_store.get_code(&email).await.map_err(|_| AuthAPIError::IncorrectCredentials)?;

    if state_login_attempt_id != request_login_attempt_id || state_two_fa_code != request_two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    two_fa_code_store.remove_code(&email).await.map_err(|_| AuthAPIError::IncorrectCredentials)?;

    let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::IncorrectCredentials)?;
    let update_jar = jar.add(auth_cookie);

    Ok((update_jar, StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    pub loginAttemptId: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String
}