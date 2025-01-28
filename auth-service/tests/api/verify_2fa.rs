use auth_service::domain::{Email, LoginAttemptId, TwoFACode};
use auth_service::utils::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let loginAtempId = LoginAttemptId::default();
    let twoFACode = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": loginAtempId,
        }),
        serde_json::json!({
            "2FACode": twoFACode,
        }),
        serde_json::json!({
            "": ""
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let loginAtempId = LoginAttemptId::default();
    let twoFACode = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "error_login_attempt_id",
            "2FACode": twoFACode
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": loginAtempId,
            "2FACode": "error_2fa_code",
        }),
        serde_json::json!({
            "email": "wrong_email",
            "loginAttemptId": loginAtempId,
            "2FACode": twoFACode
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_payload = serde_json::json!({
            "email": random_email,
            "password": "12345678",
            "requires2FA": true
        });

    app.post_signup(&signup_payload).await;

    let login_payload = serde_json::json!({
            "email": random_email,
            "password": "12345678"
        });

    app.post_login(&login_payload).await;

    let login_atemp_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let two_fa_payload = serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_atemp_id,
            "2FACode": two_fa_code
        });

    let response = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        two_fa_payload
    );
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_payload = serde_json::json!({
            "email": random_email,
            "password": "12345678",
            "requires2FA": true
        });

    app.post_signup(&signup_payload).await;

    let login_payload = serde_json::json!({
            "email": random_email,
            "password": "12345678"
        });

    app.post_login(&login_payload).await;

    let (login_attempt_id, two_fa_code);

    {
        let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
        let email = Email::parse(random_email.clone()).unwrap();
        (login_attempt_id, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    }

    let two_fa_payload = serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code
        });

    let response = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(response.status().as_u16(), 200);

    app.post_login(&login_payload).await;

    let response = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        two_fa_payload
    );
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_payload).await;

    let login_payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_payload).await;

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let (login_attempt_id, two_fa_code);
    {
        let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
        let email = Email::parse(random_email.clone()).unwrap();
        (login_attempt_id, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    }

    let two_fa_payload = serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code
        });

    std::thread::sleep(std::time::Duration::from_secs(1));

    let response = app.post_verify_2fa(&two_fa_payload).await;

    let updated_auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!updated_auth_cookie.value().is_empty());
    assert_ne!(auth_cookie.value(), updated_auth_cookie.value());
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_payload).await;

    let login_payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_payload).await;

    let (login_attempt_id, two_fa_code);
    {
        let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
        let email = Email::parse(random_email.clone()).unwrap();
        (login_attempt_id, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    }

    let two_fa_payload = serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code
        });

    let response = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(response.status().as_u16(), 401);
}