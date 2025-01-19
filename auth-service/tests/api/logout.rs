use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
   

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status().as_u16(), 200);
    
    let banned_token_store = app.app_state.banned_token_store.read().await;
    
    let token = login_response.cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .unwrap()
        .value()
        .to_string();

    let token_is_banned = banned_token_store.token_is_banned(&token).await;

    assert!(token_is_banned);

}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    app.post_signup(&signup_body).await;


    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    app.post_login(&login_body).await;

    let response = app.post_logout().await;    
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        400
    );
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        JWT_COOKIE_NAME
    );
}