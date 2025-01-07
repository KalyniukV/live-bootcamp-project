use crate::helpers::TestApp;

#[tokio::test]
async fn signup_creates_user() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let password = "password";

    let response = app.signup(email, password, true).await;

    assert_eq!(response.status().as_u16(), 200);
}