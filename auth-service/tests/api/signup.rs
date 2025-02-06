use auth_service::{routes::SignupResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};



#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123"         
        }),
        serde_json::json!({
            "123": "324"         
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(&test_case).await;
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
async fn should_return_201_if_valid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let data = serde_json::json!({
        "email": random_email,
        "password": "12345678",
        "requires2FA": true         
    });

    let response = app.post_signup(&data).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [       
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "invalidmail.com",
            "password": "password123",
            "requires2FA": true         
        }),
        serde_json::json!({
            "email": random_email,
            "password": "1234567",
            "requires2FA": true         
        })
    ];

    for test_case in test_cases.iter() {        
        let response = app.post_signup(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code  
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&payload).await;
    assert_eq!(response.status().as_u16(), 201);

    let response = app.post_signup(&payload).await;
    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );

    app.clean_up().await;
}