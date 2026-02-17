use crate::authentication::AuthenticatedUser;
use crate::domain::UpdateNote;
use actix_web::{web, HttpResponse, ResponseError};
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UpdateNoteResponse {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(thiserror::Error)]
pub enum UpdateNoteError {
    #[error("Invalid input: {0}")]
    ValidationError(String),
    #[error("Note not found")]
    NotFound,
    #[error("Invalid note ID")]
    InvalidId,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for UpdateNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for UpdateNoteError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateNoteError::ValidationError(_) => StatusCode::BAD_REQUEST,
            UpdateNoteError::NotFound => StatusCode::NOT_FOUND,
            UpdateNoteError::InvalidId => StatusCode::BAD_REQUEST,
            UpdateNoteError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Update note",
    skip(user, request, pool),
    fields(user_id = %user.user_id)
)]
pub async fn update_note(
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    request: web::Json<UpdateNoteRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UpdateNoteError> {
    let note_id = Uuid::parse_str(&note_id).map_err(|_| UpdateNoteError::InvalidId)?;

    let update = UpdateNote::parse(request.0.title, request.0.content)
        .map_err(UpdateNoteError::ValidationError)?;

    let updated_note = update_note_in_db(&pool, note_id, user.user_id, &update).await?;

    Ok(HttpResponse::Ok().json(updated_note))
}

#[tracing::instrument(name = "Update note in database", skip(pool, update))]
async fn update_note_in_db(
    pool: &PgPool,
    note_id: Uuid,
    user_id: Uuid,
    update: &UpdateNote,
) -> Result<UpdateNoteResponse, UpdateNoteError> {
    // First, check if the note exists and belongs to the user
    let existing = sqlx::query!(
        r#"
        SELECT note_id, title, content
        FROM notes
        WHERE note_id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(UpdateNoteError::NotFound)?;

    // Determine what to update
    let new_title = update
        .title
        .as_ref()
        .map(|t| t.as_ref())
        .unwrap_or(&existing.title);
    let new_content = update
        .content
        .as_ref()
        .map(|c| c.as_ref())
        .unwrap_or(&existing.content);

    // Update the note
    let row = sqlx::query!(
        r#"
        UPDATE notes
        SET title = $1, content = $2, updated_at = NOW()
        WHERE note_id = $3 AND user_id = $4
        RETURNING note_id, title, content, updated_at
        "#,
        new_title,
        new_content,
        note_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UpdateNoteResponse {
        note_id: row.note_id.to_string(),
        title: row.title,
        content: row.content,
        updated_at: row.updated_at.to_rfc3339(),
    })
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
