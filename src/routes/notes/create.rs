use crate::authentication::AuthenticatedUser;
use crate::domain::NewNote;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct CreateNoteResponse {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(thiserror::Error)]
pub enum CreateNoteError {
    #[error("Invalid input: {0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for CreateNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for CreateNoteError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateNoteError::ValidationError(_) => StatusCode::BAD_REQUEST,
            CreateNoteError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Create note",
    skip(user, request, pool),
    fields(
        user_id = %user.user_id,
        title = %request.title
    )
)]
pub async fn create_note(
    user: AuthenticatedUser,
    request: web::Json<CreateNoteRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CreateNoteError> {
    let new_note = NewNote::parse(user.user_id, request.0.title, request.0.content)
        .map_err(CreateNoteError::ValidationError)?;

    let note_id = insert_note(&pool, &new_note).await?;

    let response = CreateNoteResponse {
        note_id: note_id.to_string(),
        title: new_note.title.to_string(),
        content: new_note.content.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Created().json(response))
}

#[tracing::instrument(name = "Saving new note to database", skip(pool, new_note))]
async fn insert_note(pool: &PgPool, new_note: &NewNote) -> Result<Uuid, anyhow::Error> {
    let note_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO notes (note_id, user_id, title, content)
        VALUES ($1, $2, $3, $4)
        "#,
        note_id,
        new_note.user_id,
        new_note.title.as_ref(),
        new_note.content.as_ref(),
    )
    .execute(pool)
    .await?;

    Ok(note_id)
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
