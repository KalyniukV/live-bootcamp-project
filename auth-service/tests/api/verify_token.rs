use auth_service::utils::constants::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};


#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = TestApp::new().await;

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

    let token = login_response.cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .unwrap()
        .value()
        .to_string();

    let token_body = serde_json::json!({
        "token": token
    });
    
    let response = app.post_verify_token(&token_body).await;
    assert_eq!(response.status().as_u16(), 200);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    let token_body = serde_json::json!({
        "token": "invalid_token"
    });
    
    let response = app.post_verify_token(&token_body).await;
    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let mut app = TestApp::new().await;

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

    app.post_logout().await;

    let token = login_response.cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .unwrap()
        .value()
        .to_string();

    let token_body = serde_json::json!({
        "token": token
    });

    let verify_response = app.post_verify_token(&token_body).await;
    assert_eq!(verify_response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let body = serde_json::json!({
        "error": "qweqweqwe"
    });

    let response = app.post_login(&body).await;
    
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        body
    );

    app.clean_up().await;
}