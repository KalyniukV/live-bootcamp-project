use crate::helpers::TestApp;

#[tokio::test]
async fn verify_2fa_verifies_token() {
    let app = TestApp::new().await;

    let email = "test.mail.com";
    let login_attempt_id = "login_id";
    let code_2fa =  "some code";

    let response = app.post_verify_2fa(email, login_attempt_id, code_2fa).await;

    assert_eq!(response.status().as_u16(), 200);
}