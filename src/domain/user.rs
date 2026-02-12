use super::{UserEmail, UserPassWord};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Uuid,
    pub email: UserEmail,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updatedt_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: UserEmail,
    pub password: UserPassWord,
}

impl NewUser {
    pub fn parse(email: String, password: String) -> Result<NewUser, String> {
        let email = UserEmail::parse(email)?;
        let password = UserPassWord::parse(password)?;
        Ok(Self { email, password })
    }
}
