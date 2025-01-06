use crate::helpers::TestApp;

// Tokio's test macro is used to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn signup_creates_user() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let password = "password";

    let response = app.signup(email, password, true).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_authenticates_user() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let password = "password";

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_2fa_verifies_token() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let login_attempt_id = "login_id";
    let code_2fa =  "some code";

    let response = app.verify_2fa(email, login_attempt_id, code_2fa).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_logs_out_user() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_verifies_jwt() {
    let app = TestApp::new().await;

    let response = app.verify_token("token").await;

    assert_eq!(response.status().as_u16(), 200);
}
