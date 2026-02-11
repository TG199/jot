use crate::domain::{compute_password_hash, NewUser};
use crate::telemetry::spawn_blocking_with_tracing;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct RegistrationData {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct RegistrationResponse {
    pub user_id: Uuid,
    pub email: String,
}

#[derive(thiserror::Error)]
pub enum RegistrationError {
    #[error("Invalid input: {0}")]
    ValidationError(String),
    #[error("Email already exists")]
    EmailArealdyExists,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for RegistrationError {
    fn status_code(&self) -> StatusCode {
        match self {
            RegistrationError::ValidationError(_) => StatusCode::BAD_REQUEST,
            RegistrationError::EmailArealdyExists => StatusCode::CONFLICT,
            RegistrationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Register new user",
    skip(form, pool),
    fields(
        email = %form.email
    )
)]
pub async fn register(
    form: web::Json<RegistrationData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, RegistrationError> {
    let new_user = NewUser::parse(form.0.email.clone(), form.0.password.clone())
        .map_err(RegistrationError::ValidationError)?;

    let password_hash =
        spawn_blocking_with_tracing(move || compute_password_hash(&new_user.password))
            .await
            .context("Failed to spawn blocking task")??;

    let user_id = insert_user(&pool, &new_user.email.to_string(), &password_hash).await?;

    Ok(HttpResponse::Created().json(RegistrationResponse {
        user_id,
        email: new_user.email.to_string(),
    }))
}

#[tracing::instrument(name = "Saving new user to database", skip(pool, password_hash))]
async fn insert_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<Uuid, RegistrationError> {
    let user_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO users (user_id, email, password_hash)
        VALUES ($1, $2, $3)"#,
        user_id,
        email,
        password_hash,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        if let Some(database_error) = e.as_database_error() {
            if database_error.is_unique_violation() {
                return RegistrationError::EmailArealdyExists;
            }
        }
        RegistrationError::UnexpectedError(anyhow::anyhow!(e))
    })?;

    Ok(user_id)
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
