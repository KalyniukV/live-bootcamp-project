use auth_service::domain::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",            
        }),
        serde_json::json!({
            "email": random_email,                    
        }),
        serde_json::json!({
            "": ""         
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}


#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let data_signup = serde_json::json!({
        "email": random_email,
        "password": "12345678",
        "requires2FA": true         
    });

    app.post_signup(&data_signup).await;

    let test_cases = [
        serde_json::json!({
            "email": "",
            "password": "wrong_password"                   
        }),
        serde_json::json!({
            "email": "wrong_mail.com",
            "password": "12345678"               
        }),
    ];
    
    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}



#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let data_signup = serde_json::json!({
        "email": random_email,
        "password": "12345678",
        "requires2FA": true         
    });

    app.post_signup(&data_signup).await;

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "password": "wrong_password"
        }),
        serde_json::json!({
            "email": "anoter@mail.com",
            "password": "12345678"
        }),
    ];
    
    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}


#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": &random_email,
        "password": "password123",
        "requires2FA": true
    });

    let signup_response = app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": &random_email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;

    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    {
        let state_two_fa_code_store= app.app_state.two_fa_code_store.read().await;

        let email = Email::parse(random_email).unwrap();
        let two_fa_code_store = state_two_fa_code_store.get_code(&email).await.unwrap();

        assert_eq!(json_body.login_attempt_id, two_fa_code_store.0.as_ref());
    }

    app.clean_up().await;
}