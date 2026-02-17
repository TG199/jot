use jot::configuration::{get_configuration, DatabaseSettings};
use jot::startup::{get_connection_pool, Application};
use jot::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

// Re-export for convenience
pub use uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn test_user(&self) -> TestUser {
        self.test_user_with_email("test@example.com").await
    }

    pub async fn test_user_with_email(&self, email: &str) -> TestUser {
        let password = "ValidPass123";

        // Register the user
        let registration_body = serde_json::json!({
            "email": email,
            "password": password
        });
        let response = self.post_users(&registration_body).await;
        let user_data: serde_json::Value = response.json().await.unwrap();
        let user_id = user_data["user_id"].as_str().unwrap().to_string();

        // Login
        let login_body = serde_json::json!({
            "email": email,
            "password": password
        });
        self.post_login(&login_body).await;

        TestUser {
            user_id,
            email: email.to_string(),
        }
    }

    pub async fn post_users<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/users", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_current_user(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/users/me", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_note<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/notes", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_notes(&self, page: Option<i64>, page_size: Option<i64>) -> reqwest::Response {
        let mut url = format!("{}/notes", &self.address);
        let mut params = vec![];

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(ps) = page_size {
            params.push(format!("page_size={}", ps));
        }

        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.join("&"));
        }

        self.api_client
            .get(&url)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_note_by_id(&self, note_id: &str) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/notes/{}", &self.address, note_id))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn put_note<Body>(&self, note_id: &str, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .put(&format!("{}/notes/{}", &self.address, note_id))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn delete_note(&self, note_id: &str) -> reqwest::Response {
        self.api_client
            .delete(&format!("{}/notes/{}", &self.address, note_id))
            .send()
            .await
            .expect("Failed to execute request")
    }

    // Search and filter helpers
    pub async fn search_notes(&self, query: &str) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/notes?search={}", &self.address, query))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn filter_notes_by_date(
        &self,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> reqwest::Response {
        let mut url = format!("{}/notes", &self.address);
        let mut params = vec![];

        if let Some(f) = from {
            params.push(format!("from={}", f.to_rfc3339()));
        }
        if let Some(t) = to {
            params.push(format!("to={}", t.to_rfc3339()));
        }

        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.join("&"));
        }

        self.api_client
            .get(&url)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn sort_notes(&self, sort: &str, order: &str) -> reqwest::Response {
        self.api_client
            .get(&format!(
                "{}/notes?sort={}&order={}",
                &self.address, sort, order
            ))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn search_notes_with_filter(
        &self,
        query: &str,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> reqwest::Response {
        let mut url = format!("{}/notes?search={}", &self.address, query);

        if let Some(f) = from {
            url.push_str(&format!("&from={}", f.to_rfc3339()));
        }
        if let Some(t) = to {
            url.push_str(&format!("&to={}", t.to_rfc3339()));
        }

        self.api_client
            .get(&url)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn filter_notes_by_tag(&self, tag: &str) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/notes?tag={}", &self.address, tag))
            .send()
            .await
            .expect("Failed to execute request")
    }

    // Tag helpers
    pub async fn post_tag<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/tags", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_tags(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/tags", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> reqwest::Response {
        self.api_client
            .post(&format!(
                "{}/notes/{}/tags/{}",
                &self.address, note_id, tag_id
            ))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> reqwest::Response {
        self.api_client
            .delete(&format!(
                "{}/notes/{}/tags/{}",
                &self.address, note_id, tag_id
            ))
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub struct TestUser {
    pub user_id: String,
    pub email: String,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;

        c.redis_uri = secrecy::SecretString::new("redis://127.0.0.1:6379".into());
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let test_app = TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        api_client: client,
    };

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
