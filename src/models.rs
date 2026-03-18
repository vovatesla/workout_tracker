use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Workout {
    pub id: i64,
    pub date: String,
    pub muscle_group: String,
    pub notes: Option<String>,
    pub user_id: i64,
}

#[derive(Deserialize, Debug)]
pub struct CreateWorkoutRequest {
    pub date: String,
    pub muscle_group: String,
    pub notes: Option<String>,
}