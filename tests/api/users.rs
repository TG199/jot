use crate::helpers::{spawn_app, uuid};

#[tokio::test]
async fn register_returns_201_for_valid_form_data() {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "email": "user@example.com",
        "password": "ValidPass123"
    });

    let response = app.post_users(&body).await;

    assert_eq!(201, response.status().as_u16());

    let saved: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(saved["email"], "user@example.com");
    assert!(saved["user_id"].is_string());
}

#[tokio::test]
async fn register_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({"email": "user@example.com"}),
            "missing password",
        ),
        (
            serde_json::json!({"password": "ValidPass123"}),
            "missing email",
        ),
        (serde_json::json!({}), "missing both email and password"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_users(&invalid_body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn register_returns_400_when_email_is_invalid() {
    let app = spawn_app().await;
    let test_cases = vec![
        "invalid-email",
        "missing-at-sign.com",
        "@missing-local-part.com",
        "",
    ];

    for invalid_email in test_cases {
        let body = serde_json::json!({
            "email": invalid_email,
            "password": "ValidPass123"
        });

        let response = app.post_users(&body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return 400 Bad Request when email was '{}'.",
            invalid_email
        );
    }
}

#[tokio::test]
async fn register_returns_400_when_password_is_weak() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("short1", "too short (< 8 chars)"),
        ("nodigitshere", "no digits"),
        ("12345678", "no letters"),
        ("a".repeat(129).as_str(), "too long (> 128 chars)"),
    ];

    for (invalid_password, description) in test_cases {
        let body = serde_json::json!({
            "email": "user@example.com",
            "password": invalid_password
        });

        let response = app.post_users(&body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return 400 Bad Request when password was {}.",
            description
        );
    }
}

#[tokio::test]
async fn register_returns_409_when_email_already_exists() {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "email": "duplicate@example.com",
        "password": "ValidPass123"
    });

    // First registration should succeed
    let response = app.post_users(&body).await;
    assert_eq!(201, response.status().as_u16());

    // Second registration with same email should fail
    let response = app.post_users(&body).await;
    assert_eq!(409, response.status().as_u16());
}

#[tokio::test]
async fn register_saves_user_to_database() {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "email": "saved@example.com",
        "password": "ValidPass123"
    });

    let response = app.post_users(&body).await;
    assert_eq!(201, response.status().as_u16());

    let saved_user = sqlx::query!(
        "SELECT email, password_hash FROM users WHERE email = $1",
        "saved@example.com"
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved user.");

    assert_eq!(saved_user.email, "saved@example.com");
    assert_ne!(saved_user.password_hash, "ValidPass123");
}

#[tokio::test]
async fn register_hashes_password_properly() {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "email": "hashed@example.com",
        "password": "ValidPass123"
    });

    let response = app.post_users(&body).await;
    assert_eq!(201, response.status().as_u16());

    let saved_user = sqlx::query!(
        "SELECT password_hash FROM users WHERE email = $1",
        "hashed@example.com"
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved user.");

    assert!(saved_user.password_hash.starts_with("$argon2"));

    assert_ne!(saved_user.password_hash, "ValidPass123");
}
