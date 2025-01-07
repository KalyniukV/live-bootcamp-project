use crate::helpers::TestApp;

#[tokio::test]
async fn login_authenticates_user() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let password = "password";

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 200);
}