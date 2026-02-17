use crate::helpers::spawn_app;

#[tokio::test]
async fn delete_note_requires_authentication() {
    let app = spawn_app().await;

    let response = app
        .delete_note("550e8400-e29b-41d4-a716-446655440000")
        .await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn users_can_delete_their_own_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Note to Delete",
        "content": "This will be deleted"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    let response = app.delete_note(note_id).await;
    assert_eq!(204, response.status().as_u16());

    let get_response = app.get_note_by_id(note_id).await;
    assert_eq!(404, get_response.status().as_u16());
}

#[tokio::test]
async fn users_cannot_delete_other_users_notes() {
    let app = spawn_app().await;

    let _user1 = app.test_user().await;
    let note = serde_json::json!({
        "title": "User 1 Note",
        "content": "Protected content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id = created["note_id"].as_str().unwrap();

    app.post_logout().await;
    let _user2 = app.test_user_with_email("user2@example.com").await;

    let response = app.delete_note(note_id).await;
    assert_eq!(404, response.status().as_u16());

    app.post_logout().await;
    app.test_user().await; // Log back in as user 1
    let get_response = app.get_note_by_id(note_id).await;
    assert_eq!(200, get_response.status().as_u16());
}

#[tokio::test]
async fn delete_non_existent_note_returns_404() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let response = app
        .delete_note("550e8400-e29b-41d4-a716-446655440000")
        .await;
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn delete_with_invalid_note_id_returns_400() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let response = app.delete_note("not-a-valid-uuid").await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn deleted_note_is_removed_from_database() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "To be deleted",
        "content": "Content"
    });
    let create_response = app.post_note(&note).await;
    let created: serde_json::Value = create_response.json().await.unwrap();
    let note_id_str = created["note_id"].as_str().unwrap();
    let note_id = uuid::Uuid::parse_str(note_id_str).unwrap();

    app.delete_note(note_id_str).await;

    let result = sqlx::query!("SELECT note_id FROM notes WHERE note_id = $1", note_id)
        .fetch_optional(&app.db_pool)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn deleting_note_does_not_affect_other_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note1 = serde_json::json!({
        "title": "Note 1",
        "content": "Content 1"
    });
    let note2 = serde_json::json!({
        "title": "Note 2",
        "content": "Content 2"
    });

    let create1 = app.post_note(&note1).await;
    let created1: serde_json::Value = create1.json().await.unwrap();
    let note1_id = created1["note_id"].as_str().unwrap();

    let create2 = app.post_note(&note2).await;
    let created2: serde_json::Value = create2.json().await.unwrap();
    let note2_id = created2["note_id"].as_str().unwrap();

    app.delete_note(note1_id).await;

    let get1 = app.get_note_by_id(note1_id).await;
    assert_eq!(404, get1.status().as_u16());

    let get2 = app.get_note_by_id(note2_id).await;
    assert_eq!(200, get2.status().as_u16());
}

#[tokio::test]
async fn deleted_note_not_in_list() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note1 = serde_json::json!({
        "title": "Note 1",
        "content": "Content 1"
    });
    let note2 = serde_json::json!({
        "title": "Note 2",
        "content": "Content 2"
    });

    let create1 = app.post_note(&note1).await;
    let created1: serde_json::Value = create1.json().await.unwrap();
    let note1_id = created1["note_id"].as_str().unwrap();

    app.post_note(&note2).await;

    let list1 = app.get_notes(None, None).await;
    let body1: serde_json::Value = list1.json().await.unwrap();
    assert_eq!(body1["total_count"], 2);

    app.delete_note(note1_id).await;

    let list2 = app.get_notes(None, None).await;
    let body2: serde_json::Value = list2.json().await.unwrap();
    assert_eq!(body2["total_count"], 1);
    assert_eq!(body2["notes"][0]["title"], "Note 2");
}
