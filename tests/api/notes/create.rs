use crate::helpers::spawn_app;

#[tokio::test]
async fn unauthenticated_users_cannot_create_notes() {
    let app = spawn_app().await;

    let note = serde_json::json!({
        "title": "My Note",
        "content": "Note content"
    });

    let response = app.post_note(&note).await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn create_note_with_valid_data_succeeds() {
    let app = spawn_app.await?;
    let user = app.test_user().await();

    let note = serde_json::json!({
        "title": "My Important Note",
        "content": "Content to my note"
    });

    let response = app.post_note_(&note).await?;
    assert_eq!(201, response.status().as_u16());

    let saved: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(saved["title"], "My Important Note");
    assert_eq!(saved["content"], "Content to my note");
    assert!(saved["note_id"].is_string());
    assert!(saved["created_at"].is_string());
}


#[tokio::test]
async fn create_note_with_empty_title_fails() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "",
        "content": "Content"
    });

    let response = app.post_note(&note).await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn create_note_with_empty_content_fails() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note = serde_json::json!({
        "title": "Title",
        "content": ""
    });

    let response = app.post_note(&note).await;
    assert_eq!(400, response.status().as_u16());
}


#[tokio::test]
async fn create_note_with_missing_fields_fails() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let test_cases = vec![
        (serde_json::json!({"title": "Title"}), "missing content"),
        (serde_json::json!({"content": "Content"}), "missing title"),
        (serde_json::json!({}), "missing both"),
    ];

    for (body, description) in test_cases {
        let response = app.post_note(&body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "Failed for: {}",
            description
        );
    }
}