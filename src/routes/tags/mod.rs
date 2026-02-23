use crate::authentication::AuthenticatedUser;
use crate::domain::NewTag;
use crate::errors::user_error::TagError;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct TagResponse {
    pub tag_id: String,
    pub name: String,
}
#[tracing::instrument(name = "Create tag", skip(user, request, pool), fields(user_id = %user.user_id))]
pub async fn create_tag(
    user: AuthenticatedUser,
    request: web::Json<CreateTagRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TagError> {
    let new_tag = NewTag::parse(user.user_id, request.0.name).map_err(TagError::Validation)?;

    let tag_id = insert_tag(&pool, &new_tag).await?;

    Ok(HttpResponse::Created().json(TagResponse {
        tag_id: tag_id.to_string(),
        name: new_tag.name.to_string(),
    }))
}
#[tracing::instrument(name = "Insert tag into database", skip(pool, new_tag))]
async fn insert_tag(pool: &PgPool, new_tag: &NewTag) -> Result<Uuid, TagError> {
    let tag_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO tags (tag_id, user_id, name) VALUES ($1, $2, $3)",
        tag_id,
        new_tag.user_id,
        new_tag.name.as_ref()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        if let Some(db) = e.as_database_error() {
            if db.is_unique_violation() {
                return TagError::Duplicate;
            }
        }

        TagError::Unexpected(anyhow::anyhow!(e))
    })?;
    Ok(tag_id)
}

#[tracing::instrument(name = "List tags", skip(user, pool), fields(user_id = %user.user_id))]
pub async fn list_tags(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TagError> {
    let rows = sqlx::query!(
        "SELECT tag_id, name FROM tags WHERE user_id = $1 ORDER BY name",
        user.user_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| TagError::Unexpected(anyhow::anyhow!(e)))?;

    let tags: Vec<TagResponse> = rows
        .into_iter()
        .map(|r| TagResponse {
            tag_id: r.tag_id.to_string(),
            name: r.name,
        })
        .collect();

    Ok(HttpResponse::Ok().json(tags))
}

#[tracing::instrument(name = "Add tag to note", skip(user, pool), fields(user_id = %user.user_id))]
pub async fn add_tag_to_note(
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TagError> {
    let (note_id_str, tag_id_str) = path.into_inner();
    let note_id = Uuid::parse_str(&note_id_str).map_err(|_| TagError::InvalidId)?;
    let tag_id = Uuid::parse_str(&note_id_str).map_err(|_| TagError::InvalidId)?;

    verify_note_ownerhip(&pool, note_id, user.user_id).await?;
    verify_tag_ownerhip(&pool, tag_id, user.user_id).await?;

    sqlx::query!(
        "INSERT INTO note_tags (note_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        note_id,
        tag_id
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| TagError::Unexpected(anyhow::anyhow!(e)))?;

    Ok(HttpResponse::Created().finish())
}

#[tracing::instrument(name = "Remove tag from note", skip(user, pool), fields(user_id = %user.user_id))]
pub async fn remove_tag_from_note(
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, TagError> {
    let (note_id_str, tag_id_str) = path.into_inner();
    let note_id = Uuid::parse_str(&note_id_str).map_err(|_| TagError::InvalidId)?;
    let tag_id = Uuid::parse_str(&tag_id_str).map_err(|_| TagError::InvalidId)?;

    verify_note_ownership(&pool, note_id, user.user_id).await?;
    verify_tag_ownership(&pool, tag_id, user.user_id).await?;

    sqlx::query!(
        "DELETE FROM note_tags WHERE note_id = $1 AND tag_id = $2",
        note_id,
        tag_id
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| TagError::Unexpected(anyhow::anyhow!(e)))?;

    Ok(HttpResponse::NoContent().finish())
}

async fn verify_note_ownerhip(pool: &PgPool, note_id: Uuid, user_id: Uuid) -> Result<(), TagError> {
    sqlx::query!(
        "SELECT note_id FROM notes WHERE note_id = $1 AND user_id = $2",
        note_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| TagError::Unexpected(anyhow::anyhow!(e)))?
    .ok_or(TagError::NotFound)?;
    Ok(())
}

async fn verify_tag_ownership(pool: &PgPool, tag_id: Uuid, user_id: Uuid) -> Result<(), TagError> {
    sqlx::query!(
        "SELECT tag_id FROM tags WHERE tag_id = $1 AND user_id = $2",
        tag_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| TagError::Unexpected(anyhow::anyhow!(e)))?
    .ok_or(TagError::NotFound)?;
    Ok(())
}
