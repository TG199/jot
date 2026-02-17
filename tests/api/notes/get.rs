use crate::helpers::spawn_app;

#[tokio::test]
async fn get_note_requires_authentication() {
    let app = spawn_app().await;

    let response = app
        .get_note_by_id("550e8400-e29b-41d4-a716-446655440000")
        .await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn get_note_with_valid_id_succeeds() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Test Note",
        "content": "Test Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let response = app.get_note_by_id(note_id).await;
    assert_eq!(200, response.status().as_u16());

    let fetched: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched["note_id"], note_id);
    assert_eq!(fetched["title"], "Test Note");
    assert_eq!(fetched["content"], "Test Content");
    assert!(fetched["created_at"].is_string());
    assert!(fetched["updated_at"].is_string());
}

#[tokio::test]
async fn get_note_with_invalid_id_returns_400() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let response = app.get_note_by_id("not-a-valid-uuid").await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn get_note_that_does_not_exist_returns_404() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let response = app
        .get_note_by_id("550e8400-e29b-41d4-a716-446655440000")
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn users_cannot_get_other_users_notes() {
    let app = spawn_app().await;

    let _user1 = app.test_user().await;
    let note = serde_json::json!({
        "title": "User 1 Note",
        "content": "Private content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    app.post_logout().await;
    let _user2 = app.test_user_with_email("user2@example.com").await;

    let response = app.get_note_by_id(note_id).await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn get_note_returns_all_fields() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Complete Note",
        "content": "All fields should be present"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let response = app.get_note_by_id(note_id).await;
    let fetched: serde_json::Value = response.json().await.unwrap();

    assert!(fetched["note_id"].is_string());
    assert!(fetched["title"].is_string());
    assert!(fetched["content"].is_string());
    assert!(fetched["created_at"].is_string());
    assert!(fetched["updated_at"].is_string());
}
