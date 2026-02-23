use crate::helpers::spawn_app;

#[tokio::test]
async fn search_returns_matching_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_note(&serde_json::json!({
        "title": "Rust Programming",
        "content": "Learning about ownership and borrowing"
    }))
    .await;

    app.post_note(&serde_json::json!({
        "title": "Python Tutorial",
        "content": "Getting started with Django"
    }))
    .await;

    app.post_note(&serde_json::json!({
        "title": "JavaScript Guide",
        "content": "Understanding async/await patterns"
    }))
    .await;

    let response = app.search_notes("Jot").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 1);
    assert_eq!(body["notes"][0]["title"], "Rust Programming");
}

#[tokio::test]
async fn search_matches_in_title_and_content() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_note(&serde_json::json!({
        "title": "Shopping list",
        "content": "Buy groceries"
    }))
    .await;

    app.post_note(&serde_json::json!({
        "title": "Task list",
        "content": "Go shopping tomorrow"
    }))
    .await;

    // Search for "shopping" should find both notes
    let response = app.search_notes("shopping").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 2);
}

#[tokio::test]
async fn date_range_filtering_works() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    let start_time = chrono::Utc::now();

    app.post_note(&serde_json::json!({
        "title": "Before",
        "content": "Created before range"
    }))
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let range_start = chrono::Utc::now();

    app.post_note(&serde_json::json!({
        "title": "During",
        "content": "Created during range"
    }))
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let range_end = chrono::Utc::now();

    app.post_note(&serde_json::json!({
        "title": "After",
        "content": "Created after range"
    }))
    .await;

    // Filter for notes in range
    let response = app
        .filter_notes_by_date(Some(range_start), Some(range_end))
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 1);
    assert_eq!(body["notes"][0]["title"], "During");
}

#[tokio::test]
async fn combined_search_and_filter_works() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_note(&serde_json::json!({
        "title": "Work Task",
        "content": "Complete project"
    }))
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let cutoff = chrono::Utc::now();

    app.post_note(&serde_json::json!({
        "title": "Work Meeting",
        "content": "Team standup"
    }))
    .await;

    app.post_note(&serde_json::json!({
        "title": "Personal Note",
        "content": "Buy groceries"
    }))
    .await;

    // Search for "work" after cutoff time
    let response = app
        .search_notes_with_filter("work", Some(cutoff), None)
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 1);
    assert_eq!(body["notes"][0]["title"], "Work Meeting");
}

#[tokio::test]
async fn empty_search_returns_all_notes() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_note(&serde_json::json!({"title": "Note 1", "content": "Content 1"}))
        .await;
    app.post_note(&serde_json::json!({"title": "Note 2", "content": "Content 2"}))
        .await;

    let response = app.search_notes("").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 2);
}

#[tokio::test]
async fn search_with_no_results() {
    let app = spawn_app().await;
    let _user = app.test_user().await;

    app.post_note(&serde_json::json!({
        "title": "Rust Programming",
        "content": "Learning Rust"
    }))
    .await;

    let response = app.search_notes("Python").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["total_count"], 0);
    assert_eq!(body["notes"].as_array().unwrap().len(), 0);
}
