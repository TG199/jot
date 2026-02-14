use crate::authentication::AuthenticatedUser;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
}

#[tracing::instrument(name = "Get current user", skip(user, pool))]
pub async fn me(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_details = get_user_details(&pool, user.user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(user_details))
}

#[tracing::instrument(name = "Get user details from database", skip(pool))]
async fn get_user_details(
    pool: &PgPool,
    user_id: uuid::Uuid,
) -> Result<UserResponse, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, email
        FROM users
        WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserResponse {
        user_id: row.user_id.to_string(),
        email: row.email,
    })
}
