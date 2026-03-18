use sqlx::PgPool;
use crate::models::{Workout, CreateWorkoutRequest};
use crate::errors::AppError;

pub async fn get_all_workouts(pool: &PgPool, user_id: i64) -> Result<Vec<Workout>, AppError> {
    sqlx::query_as!(Workout, "SELECT * FROM workouts WHERE user_id = $1", user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::DatabaseError)
}

pub async fn get_workout_by_id(pool: &PgPool, id: i64, user_id: i64) -> Result<Option<Workout>, AppError> {
    sqlx::query_as!(
        Workout,
        "SELECT * FROM workouts WHERE id = $1 AND user_id = $2",
        id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::DatabaseError)
}

pub async fn create_workout(pool: &PgPool, payload: CreateWorkoutRequest, user_id: i64) -> Result<Workout, AppError> {
    sqlx::query_as!(
        Workout,
        "INSERT INTO workouts (date, muscle_group, notes, user_id) VALUES ($1, $2, $3, $4) RETURNING *",
        payload.date,
        payload.muscle_group,
        payload.notes,
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::DatabaseError)
}

pub async fn update_workout(pool: &PgPool, id: i64, payload: CreateWorkoutRequest, user_id: i64) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "UPDATE workouts SET date = $1, muscle_group = $2, notes = $3 WHERE id = $4 AND user_id = $5",
        payload.date,
        payload.muscle_group,
        payload.notes,
        id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_workout(pool: &PgPool, id: i64, user_id: i64) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM workouts WHERE id = $1 AND user_id = $2",
        id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::DatabaseError)?;
    Ok(())
}