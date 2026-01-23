use jot::configuration::{get_configuration, DatabaseSettings};
use jot::startup::{get_configuration_pool, Application};
use jot::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use reqwest::Body;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::fmt::format;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subcriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
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

    pub async fn get_notes(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/notes", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_note(&self, note_id: &Uuid) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/notes/{}", &self.address, note_id))
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address,
        port: application_port,
        db_pool: get_configuration_pool(&configuration.database),
        api_client: client,
    };
    test_app
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}"; "#, config.database_name).as_str())
        .await
        .expect("Failed to create Database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to migrate the database");
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
