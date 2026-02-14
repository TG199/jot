use crate::configuration::DatabaseSettings;
use crate::configuration::Settings;
use crate::routes::health_check;
use crate::routes::home;
use crate::routes::login;
use crate::routes::logout;
use crate::routes::me;
use crate::routes::register;
use crate::session_state::session_middleware;

use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use reqwest::Url;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            configuration.application.url().expect("Invalid host url"),
            configuration.application.hmac_secret,
            configuration.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub Url);

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    base_url: Url,
    hmac_secret: SecretString,
    redis_uri: SecretString,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    // Create Redis session store
    let session_mw = session_middleware(redis_uri, secret_key.clone()).await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(session_mw.clone())
            .wrap(message_framework.clone())
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/health", web::get().to(health_check))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/users", web::post().to(register))
            .route("/users/me", web::get().to(me))
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone, Deserialize)]
pub struct HmacSecret(pub SecretString);
