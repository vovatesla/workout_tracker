use axum::{extract::{State, Path, Extension}, Json};
use axum::http::StatusCode;
use sqlx::PgPool;
use redis::Client as RedisClient;
use lapin::Channel;
use tracing::info;
use crate::models::{Workout, CreateWorkoutRequest};
use crate::errors::AppError;
use crate::repository;
use crate::cache;
use crate::queue::{self, WorkoutCreatedEvent};

type AppState = (PgPool, String, RedisClient, Channel);

pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn ping_handler() -> &'static str {
    "pong"
}

pub async fn list_workouts(
    State((pool, _, redis, _)): State<AppState>,
    Extension(user_id_str): Extension<String>,
) -> Result<Json<Vec<Workout>>, AppError> {
    let user_id: i64 = user_id_str.parse().map_err(|_| AppError::NotFound)?;
    let cache_key = format!("workouts:user:{}", user_id);

    if let Some(cached) = cache::get(&redis, &cache_key).await {
        let workouts: Vec<Workout> = serde_json::from_str(&cached)
            .map_err(|_| AppError::NotFound)?;
        return Ok(Json(workouts));
    }

    let workouts = repository::get_all_workouts(&pool, user_id).await?;
    let serialized = serde_json::to_string(&workouts).unwrap_or_default();
    cache::set(&redis, &cache_key, &serialized, 60).await;

    Ok(Json(workouts))
}

pub async fn create_workout_handler(
    State((pool, _, redis, rabbit)): State<AppState>,
    Extension(user_id_str): Extension<String>,
    Json(payload): Json<CreateWorkoutRequest>,
) -> Result<Json<Workout>, AppError> {
    let user_id: i64 = user_id_str.parse().map_err(|_| AppError::NotFound)?;
    let workout = repository::create_workout(&pool, payload, user_id).await?;
    info!("Создан воркаут id={}", workout.id);

    let cache_key = format!("workouts:user:{}", user_id);
    cache::delete(&redis, &cache_key).await;

    let event = WorkoutCreatedEvent {
        workout_id: workout.id,
        user_id,
        muscle_group: workout.muscle_group.clone(),
    };
    match queue::publish(&rabbit, "workout_created", &event).await {
    Ok(_) => info!("Событие отправлено в очередь workout_created"),
    Err(e) => info!("Не удалось отправить событие: {}", e),
}

    Ok(Json(workout))
}

pub async fn get_workout(
    State((pool, _, _, _)): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id_str): Extension<String>,
) -> Result<Json<Workout>, AppError> {
    let user_id: i64 = user_id_str.parse().map_err(|_| AppError::NotFound)?;
    let workout = repository::get_workout_by_id(&pool, id, user_id).await?;
    match workout {
        Some(w) => Ok(Json(w)),
        None => Err(AppError::NotFound),
    }
}

pub async fn put_workout(
    State((pool, _, redis, _)): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id_str): Extension<String>,
    Json(payload): Json<CreateWorkoutRequest>,
) -> Result<StatusCode, AppError> {
    let user_id: i64 = user_id_str.parse().map_err(|_| AppError::NotFound)?;
    let updated = repository::update_workout(&pool, id, payload, user_id).await?;

    let cache_key = format!("workouts:user:{}", user_id);
    cache::delete(&redis, &cache_key).await;

    if updated {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn delete_workout(
    State((pool, _, redis, _)): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id_str): Extension<String>,
) -> Result<StatusCode, AppError> {
    let user_id: i64 = user_id_str.parse().map_err(|_| AppError::NotFound)?;
    repository::delete_workout(&pool, id, user_id).await?;

    let cache_key = format!("workouts:user:{}", user_id);
    cache::delete(&redis, &cache_key).await;

    Ok(StatusCode::OK)
}