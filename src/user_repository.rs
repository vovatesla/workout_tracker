use sqlx::PgPool;
use crate::errors::AppError;

pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<i64, AppError> {
    let row = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id",
        username,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(row.id)
}

pub async fn find_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(user)
}