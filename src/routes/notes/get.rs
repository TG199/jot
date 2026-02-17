use crate::authentication::AuthenticatedUser;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct NoteResponse {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(thiserror::Error)]
pub enum GetNoteError {
    #[error("Note not found")]
    NotFound,
    #[error("Invalid note ID")]
    InvalidId,
    #[error("transparent")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for GetNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for GetNoteError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetNoteError::NotFound => StatusCode::NOT_FOUND,
            GetNoteError::InvalidId => StatusCode::BAD_REQUEST,
            GetNoteError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Get note", skip(user, pool))]
pub async fn get_note(
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, GetNoteError> {
    let note_id = Uuid::parse_str(&note_id).map_err(|_| GetNoteError::InvalidId)?;

    let note = fetch_note(&pool, note_id, user.user_id).await?;

    Ok(HttpResponse::Ok().json(note))
}

#[tracing::instrument(name = "Fetch note from database", skip(pool))]
async fn fetch_note(
    pool: &PgPool,
    note_id: Uuid,
    user_id: Uuid,
) -> Result<NoteResponse, GetNoteError> {
    let row = sqlx::query!(
        r#"
        SELECT note_id, title, content, created_at, updated_at
        FROM notes
        WHERE note_id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(GetNoteError::NotFound)?;

    Ok(NoteResponse {
        note_id: row.note_id.to_string(),
        title: row.title,
        content: row.content,
        created_at: row.created_at.to_rfc3339(),
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
