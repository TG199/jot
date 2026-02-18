use crate::authentication::AuthenticatedUser;
use actix_web::{web, Error, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder};

#[derive(Deserialize)]
pub struct NoteQueryParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub search: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default = "default_order")]
    pub order: String,
    pub tag: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

fn default_sort() -> String {
    "created_at".to_string()
}

fn default_order() -> String {
    "desc".to_string()
}

#[derive(Serialize)]
pub struct NoteListItem {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct NoteListeResponse {
    pub notes: Vec<NoteListItem>,
    pub page: i64,
    pub page_size: i64,
    pub total_count: i64,
}

#[tracing::instrument(name = "List user notes", skip(user, pool, params))]
pub async fn list_notes(
    user: AuthenticatedUser,
    params: web::Query<NoteQueryParams>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let sorted_field = validate_sort_field(&params.sort);
    let order = if params.order.to_lowercase() == "asc" {
        "ASC"
    } else {
        "DESC"
    };

    let total_count = get_notes_count(
        &pool,
        user.user_id,
        &params.search,
        &params.from,
        &params.to,
        &params.tag,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let notes = get_notes(
        &pool,
        user.user_id,
        page_size,
        offset,
        &params.search,
        &params.from,
        &params.to,
        sorted_field,
        order,
        &params.tag,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(NoteListeResponse {
        notes,
        page,
        page_size,
        total_count,
    }))
}

fn validate_sort_field(field: &str) -> &str {
    match field {
        "title" => "title",
        "created_at" => "created_at",
        "updated_at" => "updated_at",
        _ => "created_at",
    }
}
#[tracing::instrument(name = "Get notes count from database", skip(pool))]
async fn get_notes_count(
    pool: &PgPool,
    user_id: uuid::Uuid,
    search: &Option<String>,
    from: &Option<DateTime<Utc>>,
    to: &Option<DateTime<Utc>>,
    tag: &Option<String>,
) -> Result<i64, anyhow::Error> {
    let mut query = sqlx::QueryBuilder::new("SELECT COUNT(DISTINCT n.note_id) FROM notes n");

    if tag.is_some() {
        query.push(" LEFT JOIN note_tags nt ON n.note_id = nt.note_id");
        query.push(" LEFT JOIN tags t ON nt.tag_id = t.tag_id");
    }
    query.push(" WHERE n.user_id = ");
    query.push_bind(user_id);

    if let Some(search_term) = search {
        if !search_term.is_empty() {
            query.push(" AND (");
            query.push("to_tsvector('english', n.title || ' ' || n.content) @@ plainto_tsquery('english', ");
            query.push_bind(search_term);
            query.push(")");
        }
    }

    if let Some(from_date) = from {
        query.push(" AND n.created_at >= ");
        query.push_bind(from_date);
    }

    if let Some(to_date) = to {
        query.push(" AND n.created_at <= ");
        query.push_bind(to_date);
    }

    if let Some(tag_name) = tag {
        query.push(" AND t.name = ");
        query.push_bind(tag_name);
    }

    let row: (i64,) = query.build_query_as().fetch_one(pool).await?;
    Ok(row.0)
}

#[tracing::instrument(name = "Get notes from database", skip(pool))]
async fn get_notes(
    pool: &PgPool,
    user_id: uuid::Uuid,
    limit: i64,
    offset: i64,
    search: &Option<String>,
    from: &Option<DateTime<Utc>>,
    to: &Option<DateTime<Utc>>,
    sort_field: &str,
    order: &str,
    tag: &Option<String>,
) -> Result<Vec<NoteListItem>, anyhow::Error> {
    let mut query = QueryBuilder::new(
        "SELECT DISTINCT n.note_id, n.title n.content, n.created_at, n.updated_at FROM notes n",
    );

    if tag.is_some() {
        query.push(" LEFT JOIN note_tags nt ON n.note_id = nt.note_id");
        query.push(" LEFT JOIN tags t ON nt.tag_id = t.tag_id");
    }

    query.push(" WHERE n.user_id = ");
    query.push_bind(user_id);

    if let Some(search_term) = search {
        if !search_term.is_empty() {
            query.push(" AND (");
            query.push("to_tsvector('english', n.title || ' '  || n.content) @@ plainto_tsquery('english', ");
            query.push_bind(search_term);
            query.push(")");
        }
    }

    if let Some(from_date) = from {
        query.push(" AND n.created_at >= ");
        query.push_bind(from_date);
    }

    if let Some(to_date) = to {
        query.push(" AND n.created_at <= ");
        query.push_bind(to_date);
    }

    if let Some(tag_name) = tag {
        query.push(" AND t.name = ");
        query.push_bind(tag_name);
    }

    query.push(format!(" ORDER BY n.{} {}", sort_field, order));

    query.push(" LIMIT ");
    query.push_bind(limit);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let rows = query
        .build_query_as::<(uuid::Uuid, String, String, DateTime<Utc>, DateTime<Utc>)>()
        .fetch_all(pool)
        .await?;

    let mut notes = Vec::new();

    for row in rows {
        let tags = get_note_tags(pool, row.0).await?;

        notes.push(NoteListItem {
            note_id: row.0.to_string(),
            title: row.1,
            content: row.2,
            created_at: row.3.to_rfc3339(),
            updated_at: row.4.to_rfc3339(),
            tags: if tags.is_empty() { None } else { Some(tags) },
        });
    }

    Ok(notes)
}

#[tracing::instrument(name = "Get tags for note", skip(pool))]
async fn get_note_tags(pool: &PgPool, note_id: uuid::Uuid) -> Result<Vec<String>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT t.name
        FROM tags t
        JOIN note_tags nt ON t.tag_id = nt.tag_id
        WHERE nt.note_id = $1
        ORDER BY t.name
        "#,
        note_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.name).collect())
}
