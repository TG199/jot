use crate::helpers::spawn_app;

#[tokio::test]
async fn list_notes_requires_authentication() {
    let app = spawn_app().await;

    let response = app.get_notes(None, None).await;
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn users_can_only_see_their_own_notes() {
    let app = spawn_app().await;

    let user1 = app.test_user().await;
    let note1 = serde_json::json!({
        "title": "User 1 Note",
        "content": "Content from user 1"
    });
    app.post_note(&note1).await;

    app.post_logout().await;
    let user2 = app.test_user_with_email("user2@example.com").await;
    let note2 = serde_json::json!({
        "title": "User 2 Note",
        "content": "Content from user 2"
    });
    app.post_note(&note2).await;

    let response = app.get_notes(None, None).await;
    assert_eq!(200, response.status().as_u16());

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 1);
    assert_eq!(body["notes"][0]["title"], "User 2 Note");
}

#[tokio::test]
async fn list_notes_returns_empty_array_when_user_has_no_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let response = app.get_notes(None, None).await;
    assert_eq!(200, response.status().as_u16());

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 0);
    assert_eq!(body["notes"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_notes_returns_all_user_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    // Create multiple notes
    for i in 1..=3 {
        let note = serde_json::json!({
            "title": format!("Note {}", i),
            "content": format!("Content {}", i)
        });
        app.post_note(&note).await;
    }

    let response = app.get_notes(None, None).await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["total_count"], 3);
    assert_eq!(body["notes"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn pagination_works_correctly() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    for i in 1..=25 {
        let note = serde_json::json!({
            "title": format!("Note {}", i),
            "content": format!("Content {}", i)
        });
        app.post_note(&note).await;
    }

    // Get first page (default page size is 20)
    let response = app.get_notes(Some(1), Some(20)).await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 25);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 20);
    assert_eq!(body["notes"].as_array().unwrap().len(), 20);

    // Get second page
    let response = app.get_notes(Some(2), Some(20)).await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 25);
    assert_eq!(body["page"], 2);
    assert_eq!(body["page_size"], 20);
    assert_eq!(body["notes"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn pagination_handles_page_size_limits() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    // Create one note
    let note = serde_json::json!({
        "title": "Test Note",
        "content": "Content"
    });
    app.post_note(&note).await;

    // Request with page size > 100 should be clamped to 100
    let response = app.get_notes(Some(1), Some(200)).await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["page_size"], 100);

    // Request with page size < 1 should be clamped to 1
    let response = app.get_notes(Some(1), Some(0)).await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["page_size"], 1);
}
