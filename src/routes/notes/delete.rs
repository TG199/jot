use crate::authentication::AuthenticatedUser;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum DeleteNoteError {
    #[error("Note not found")]
    NotFound,
    #[error("Invalid note ID")]
    InvalidId,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for DeleteNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for DeleteNoteError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteNoteError::NotFound => StatusCode::NOT_FOUND,
            DeleteNoteError::InvalidId => StatusCode::BAD_REQUEST,
            DeleteNoteError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Delete note", skip(user, pool))]
pub async fn delete_note(
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, DeleteNoteError> {
    let note_id = Uuid::parse_str(&note_id).map_err(|_| DeleteNoteError::InvalidId)?;

    delete_note_from_db(&pool, note_id, user.user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[tracing::instrument(name = "Delete note from database", skip(pool))]
async fn delete_note_from_db(
    pool: &PgPool,
    note_id: Uuid,
    user_id: Uuid,
) -> Result<(), DeleteNoteError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM notes
        WHERE note_id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DeleteNoteError::NotFound);
    }

    Ok(())
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
