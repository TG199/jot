use crate::helpers::spawn_app;


#[tokio::test]
async fn login_with_valid_credentials_succeeds() {
    let app = spawn_app().await();

    let registration_body = serde_json::json!({
        "email": "jot@example.com",
        "password": "jotpass1234"
    });

    let response = app.post_users(&registration_body).await;
    assert_eq!(201, response.status().as_u16());

    let login_body = serde_json::json!({
        "email": "jot@example.com",
        "password": "jotpass1234"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(200, response.status().as_u16());

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["message"], "Login successful");

    assert!(body["user_id"].is_string());
}


#[tokio::test]
async fn login_with_wrong_password_fails() {
    let app = spawn_app().await;

    let registration_body = serde_json::json!({
        "email": "jot@example.com",
        "password": "jotPass123"
    });
    app.post_users(&registration_body).await;

    let login_body = serde_json::json!({
        "email": "jot@example.com",
        "password": "jotWrongPass123"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn login_with_unknown_email_fails() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "email": "unknown@example.com",
        "password": "SomePass123"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn login_with_missing_credentials_returns_400() {
    let app = spawn_app().await;

    let test_cases = vec![
        (serde_json::json!({"email": "user@example.com"}), "missing password"),
        (serde_json::json!({"password": "ValidPass123"}), "missing email"),
        (serde_json::json!({}), "missing both"),
    ];

    for (body, description) in test_cases {
        let response = app.post_login(&body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "Failed for case: {}",
            description
        );
    }
}

#[tokio::test]
async fn session_persists_across_requests() {
    let app = spawn_app().await;

    let registration_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    app.post_users(&registration_body).await;

    let login_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(200, login_response.status().as_u16());

    let cookies = login_response.cookies().collect::<Vec<_>>();
    assert!(!cookies.is_empty(), "No session cookie set");

    let me_response = app.get_current_user().await;
    assert_eq!(200, me_response.status().as_u16());

    let user: serde_json::Value = me_response.json().await.expect("Failed to parse response");
    assert_eq!(user["email"], "user@example.com");
}

#[tokio::test]
async fn logout_invalidates_session() {
    let app = spawn_app().await;

    let registration_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    app.post_users(&registration_body).await;

    let login_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    app.post_login(&login_body).await;

    let me_response = app.get_current_user().await;
    assert_eq!(200, me_response.status().as_u16());

    let logout_response = app.post_logout().await;
    assert_eq!(200, logout_response.status().as_u16());

    let me_response_after_logout = app.get_current_user().await;
    assert_eq!(401, me_response_after_logout.status().as_u16());
}

#[tokio::test]
async fn protected_endpoint_requires_authentication() {
    let app = spawn_app().await;

    let response = app.get_current_user().await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn multiple_users_can_login_independently() {
    let app = spawn_app().await;

    let user1_reg = serde_json::json!({
        "email": "user1@example.com",
        "password": "Password123"
    });
    let user2_reg = serde_json::json!({
        "email": "user2@example.com",
        "password": "Password456"
    });

    app.post_users(&user1_reg).await;
    app.post_users(&user2_reg).await;

    let login1 = serde_json::json!({
        "email": "user1@example.com",
        "password": "Password123"
    });
    let response1 = app.post_login(&login1).await;
    assert_eq!(200, response1.status().as_u16());

    let app2 = spawn_app().await;

    let login2 = serde_json::json!({
        "email": "user2@example.com",
        "password": "Password456"
    });
    let response2 = app2.post_login(&login2).await;
    assert_eq!(200, response2.status().as_u16());
}

#[tokio::test]
async fn session_is_renewed_on_login() {
    let app = spawn_app().await;

    let registration_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    app.post_users(&registration_body).await;

    let login_body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });
    let response = app.post_login(&login_body).await;

    let cookies = response.cookies().collect::<Vec<_>>();
    assert!(!cookies.is_empty(), "Session cookie should be set on login");
}