use crate::helpers::spawn_app;

#[tokio::test]
async fn create_tag_succeeds() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let tag = serde_json::json!({"name": "work"});
    let response = app.post_tag(&tag).await;

    assert_eq!(201, response.status().as_u16());
    let created: serde_json::Value = response.json().await.unwrap();
    assert_eq!(created["name"], "work");
    assert!(created["tag_id"].is_string());
}

#[tokio::test]
async fn duplicate_tag_creation_fails() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let tag = serde_json::json!({"name": "work"});
    app.post_tag(&tag).await;

    // Try to create same tag again
    let response = app.post_tag(&tag).await;
    assert_eq!(409, response.status().as_u16());
}

#[tokio::test]
async fn invalid_tag_names_are_rejected() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let test_cases = vec![
        ("", "empty"),
        ("   ", "whitespace"),
        ("tag name", "contains space"),
        ("tag@name", "contains @"),
        ("tag!name", "contains !"),
        ("a".repeat(51).as_str(), "too long"),
    ];

    for (name, description) in test_cases {
        let tag = serde_json::json!({"name": name});
        let response = app.post_tag(&tag).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "Should reject: {}",
            description
        );
    }
}

#[tokio::test]
async fn valid_tag_names_with_special_chars() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let valid_names = vec!["work", "my-tag", "tag_123", "tag-with-hyphens"];

    for name in valid_names {
        let tag = serde_json::json!({"name": name});
        let response = app.post_tag(&tag).await;
        assert_eq!(201, response.status().as_u16(), "Should accept: {}", name);
    }
}

#[tokio::test]
async fn list_tags_returns_user_tags() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_tag(&serde_json::json!({"name": "work"})).await;
    app.post_tag(&serde_json::json!({"name": "personal"})).await;
    app.post_tag(&serde_json::json!({"name": "urgent"})).await;

    let response = app.get_tags().await;
    assert_eq!(200, response.status().as_u16());

    let tags: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(tags.len(), 3);
}

#[tokio::test]
async fn tags_are_sorted_alphabetically() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_tag(&serde_json::json!({"name": "zebra"})).await;
    app.post_tag(&serde_json::json!({"name": "apple"})).await;
    app.post_tag(&serde_json::json!({"name": "monkey"})).await;

    let response = app.get_tags().await;
    let tags: Vec<serde_json::Value> = response.json().await.unwrap();

    assert_eq!(tags[0]["name"], "apple");
    assert_eq!(tags[1]["name"], "monkey");
    assert_eq!(tags[2]["name"], "zebra");
}

#[tokio::test]
async fn add_tag_to_note_succeeds() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note_response = app
        .post_note(&serde_json::json!({
            "title": "Work Note",
            "content": "Important task"
        }))
        .await;
    let note: serde_json::Value = note_response.json().await.unwrap();
    let note_id = note["note_id"].as_str().unwrap();

    let tag_response = app.post_tag(&serde_json::json!({"name": "work"})).await;
    let tag: serde_json::Value = tag_response.json().await.unwrap();
    let tag_id = tag["tag_id"].as_str().unwrap();

    let response = app.add_tag_to_note(note_id, tag_id).await;
    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn remove_tag_from_note_succeeds() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note_response = app
        .post_note(&serde_json::json!({
            "title": "Work Note",
            "content": "Important task"
        }))
        .await;
    let note: serde_json::Value = note_response.json().await.unwrap();
    let note_id = note["note_id"].as_str().unwrap();

    let tag_response = app.post_tag(&serde_json::json!({"name": "work"})).await;
    let tag: serde_json::Value = tag_response.json().await.unwrap();
    let tag_id = tag["tag_id"].as_str().unwrap();

    app.add_tag_to_note(note_id, tag_id).await;
    let response = app.remove_tag_from_note(note_id, tag_id).await;
    assert_eq!(204, response.status().as_u16());
}

#[tokio::test]
async fn adding_same_tag_twice_is_idempotent() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let note_response = app
        .post_note(&serde_json::json!({
            "title": "Note",
            "content": "Content"
        }))
        .await;
    let note: serde_json::Value = note_response.json().await.unwrap();
    let note_id = note["note_id"].as_str().unwrap();

    let tag_response = app.post_tag(&serde_json::json!({"name": "work"})).await;
    let tag: serde_json::Value = tag_response.json().await.unwrap();
    let tag_id = tag["tag_id"].as_str().unwrap();

    app.add_tag_to_note(note_id, tag_id).await;
    let response = app.add_tag_to_note(note_id, tag_id).await;
    assert_eq!(201, response.status().as_u16());

    let list_response = app.get_notes(None, None).await;
    let body: serde_json::Value = list_response.json().await.unwrap();
    let tags = body["notes"][0]["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 1);
}
