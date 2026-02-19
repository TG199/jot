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
    
)
