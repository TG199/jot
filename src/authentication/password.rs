use crate::domain::UserPassWord;
use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub email: String,
    pub password: SecretString,
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = SecretString::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .into(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.email, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown email."))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Get stored credentials", skip(email, pool))]
async fn get_stored_credentials(
    email: &str,
    pool: &PgPool,
) -> Result<Option<(Uuid, SecretString)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE email = $1"#,
        email,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?
    .map(|row| (row.user_id, SecretString::new(row.password_hash.into())));

    Ok(row)
}

impl Credentials {
    pub fn new(email: String, password: UserPassword) -> Self {
        Self {
            email,
            password: SecretString::new(password.expose_secret().to_string().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_with_valid_hash_succeeds() {
        let password = SecretString::new("TestPassword123".into());
        // This is the hash for "TestPassword123" - generated once for testing
        let hash = SecretString::new(
            "$argon2id$v=19$m=19456,t=2,p=1$\
            VE0dJ1xo8RcJKxCc4KXMqw$\
            qmM+1JDsIGMjGGOEUBGJrKCfVLGzCCZGbXGxoLUzEq4"
                .into(),
        );

        let result = verify_password_hash(hash, password);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_password_with_invalid_password_fails() {
        let password = SecretString::new("WrongPassword123".into());
        let hash = SecretString::new(
            "$argon2id$v=19$m=19456,t=2,p=1$\
            VE0dJ1xo8RcJKxCc4KXMqw$\
            qmM+1JDsIGMjGGOEUBGJrKCfVLGzCCZGbXGxoLUzEq4"
                .into(),
        );

        let result = verify_password_hash(hash, password);
        assert!(result.is_err());
    }
}
