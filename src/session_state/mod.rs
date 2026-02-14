use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use secrecy::ExposeSecret;
use secrecy::SecretString;

pub fn session_middleware(
    redis_uri: SecretString,
    secret_key: Key,
) -> Result<SessionMiddleware<RedisSessionStore>, anyhow::Error> {
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret())
        .map_err(|e| anyhow::anyhow!("Failed to  connect to Redis: {}", e))?;

    Ok(SessionMiddleware::new(redis_store, secret_key))
}
