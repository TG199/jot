use crate::helpers::spawn_app;

#[tokio::test]
async fn update_note_requires_authentication() {
    let app = spawn_app().await;

    let update = serde_json::json!({
        "title": "Updated Title"
    });

    let response = app
        .put_note("550e8400-e29b-41d4-a716-446655440000", &update)
        .await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn users_can_update_their_own_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Original Title",
        "content": "Original Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let update = serde_json::json!({
        "title": "Updated Title",
        "content": "Updated Content"
    });
    let response = app.put_note(note_id, &update).await;
    assert_eq!(200, response.status().as_u16());

    let updated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated["title"], "Updated Title");
    assert_eq!(updated["content"], "Updated Content");
    assert_eq!(updated["note_id"], note_id);
}

#[tokio::test]
async fn users_cannot_update_other_users_notes() {
    let app = spawn_app().await;

    let _user1 = app.test_user().await;
    let note = serde_json::json!({
        "title": "User 1 Note",
        "content": "User 1 Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    app.post_logout().await;
    let _user2 = app.test_user_with_email("user2@example.com").await;

    let update = serde_json::json!({
        "title": "Malicious Update"
    });
    let response = app.put_note(note_id, &update).await;
    assert_eq!(404, response.status().as_u16());

    app.post_logout().await;
    app.test_user().await; // Log back in as user 1
    let get_response = app.get_note_by_id(note_id).await;
    let note: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(note["title"], "User 1 Note");
}

#[tokio::test]
async fn update_non_existent_note_returns_404() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let update = serde_json::json!({
        "title": "New Title"
    });

    let response = app
        .put_note("550e8400-e29b-41d4-a716-446655440000", &update)
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn partial_update_title_only_works() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Original Title",
        "content": "Original Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let update = serde_json::json!({
        "title": "New Title"
    });
    let response = app.put_note(note_id, &update).await;
    assert_eq!(200, response.status().as_u16());

    let updated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated["title"], "New Title");
    assert_eq!(updated["content"], "Original Content");
}

#[tokio::test]
async fn partial_update_content_only_works() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Original Title",
        "content": "Original Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let update = serde_json::json!({
        "content": "New Content"
    });
    let response = app.put_note(note_id, &update).await;
    assert_eq!(200, response.status().as_u16());

    let updated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated["title"], "Original Title"); // Title unchanged
    assert_eq!(updated["content"], "New Content");
}

#[tokio::test]
async fn update_with_no_fields_fails() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Original Title",
        "content": "Original Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let update = serde_json::json!({});
    let response = app.put_note(note_id, &update).await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn update_with_invalid_note_id_returns_400() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let update = serde_json::json!({
        "title": "New Title"
    });

    let response = app.put_note("not-a-valid-uuid", &update).await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn updated_at_changes_after_update() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Original Title",
        "content": "Original Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();
    let original_updated_at = created["created_at"].as_str().unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let update = serde_json::json!({
        "title": "Updated Title"
    });
    let response = app.put_note(note_id, &update).await;
    let updated: serde_json::Value = response.json().await.unwrap();
    let new_updated_at = updated["updated_at"].as_str().unwrap();

    assert_ne!(original_updated_at, new_updated_at);
}
