use crate::authentication::AuthenticatedUser;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{any, PgPool};

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Serialize)]
pub struct NoteListItem {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct NoteListeResponse {
    pub notes: Vec<NoteListItem>,
    pub page: i64,
    pub page_size: i64,
    pub total_count: i64,
}

#[tracing::instrument(name = "List user notes", skip(user, pool))]
pub async fn list_notes(
    user: AuthenticatedUser,
    params: web::Query<PaginationParams>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total_count = get_notes_count(&pool, user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = NoteListeResponse {
        notes,
        page,
        page_size,
        total_count,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(name = "Get notes count from database", skip(pool))]
async fn get_notes_count(pool: &PgPool, user_id: uuid::Uuid) -> Result<i64, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM notes
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.count.unwrap_or(0))
}
#[tracing::instrument(name = "Get notes from database", skip(pool))]
async fn get_notes(
    pool: &PgPool,
    user_id: uuid::Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<NoteListItem>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT note_id, title, content, created_at, updated_at
        FROM notes
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    let notes = rows
        .into_iter()
        .map(|row| NoteListItem {
            note_id: row.note_id.to_string(),
            title: row.title,
            content: row.content,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(notes)
}
