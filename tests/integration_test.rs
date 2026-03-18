use axum::{
    routing::{get, post, put, delete},
    Router,
};
use axum::middleware as axum_middleware;
use sqlx::PgPool;
use tokio::net::TcpListener;
use workout_tracker::handlers;
use workout_tracker::auth_handlers;
use workout_tracker::middleware::require_auth;
use workout_tracker::config::Config;
use workout_tracker::cache;
use workout_tracker::queue;

async fn spawn_app() -> String {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let pool = PgPool::connect(&config.database_url).await.unwrap();

    let redis_client = cache::create_client(&config.redis_url).expect("Failed to create Redis client");

    let rabbit_connection = queue::create_connection(&config.rabbitmq_url).await.expect("Failed to connect to RabbitMQ");
    let rabbit_channel = queue::create_channel(&rabbit_connection).await.expect("Failed to create RabbitMQ channel");
    queue::declare_queue(&rabbit_channel, "workout_created").await.expect("Failed to declare queue");

    let state = (pool.clone(), config.jwt_secret.clone(), redis_client, rabbit_channel);

    let protected = Router::new()
        .route("/workouts", get(handlers::list_workouts))
        .route("/workouts", post(handlers::create_workout_handler))
        .route("/workouts/:id", get(handlers::get_workout))
        .route("/workouts/:id", put(handlers::put_workout))
        .route("/workouts/:id", delete(handlers::delete_workout))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    let app = Router::new()
        .route("/health", get(handlers::health_handler))
        .route("/ping", get(handlers::ping_handler))
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        .merge(protected)
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn health_check_works() {
    let base_url = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ping_check_works() {
    let base_url = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/ping", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = response.text().await.unwrap();
    assert_eq!(body, "pong");
}

#[tokio::test]
async fn create_workout_check_works() {
    let base_url = spawn_app().await;
    let token = get_token(&base_url).await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/workouts", base_url))
        .header("Authorization", format!("Bearer {}", token))  // добавляем токен
        .json(&serde_json::json!({
            "date": "2026-03-06",
            "muscle_group": "chest",
            "notes": "bench 80x5"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["id"].is_number());
    assert_eq!(body["muscle_group"], "chest");
}

#[tokio::test]
async fn get_workout_check_works() {
    let base_url = spawn_app().await;
    let token = get_token(&base_url).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/workouts/999", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn delete_workout_works() {
    let base_url = spawn_app().await;
    let token = get_token(&base_url).await;

    let client = reqwest::Client::new();

    let create_response = client
    .post(format!("{}/workouts", base_url))
    .header("Authorization", format!("Bearer {}", token))
    .json(&serde_json::json!({
        "date": "2026-03-06",
        "muscle_group": "chest",
        "notes": "bench 80x5"
    }))
    .send()
    .await
    .unwrap();

    let workout: serde_json::Value = create_response.json().await.unwrap();
    let id = workout["id"].as_i64().unwrap();

    let delete_response = client
    .delete(format!("{}/workouts/{}", base_url, id))
    .header("Authorization", format!("Bearer {}", token))
    .send()
    .await
    .unwrap();

    assert_eq!(delete_response.status(), 200);
}

async fn get_token(base_url: &str) -> String {
    let client = reqwest::Client::new();

    client
        .post(format!("{}/register", base_url))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .unwrap();

    let response = client
        .post(format!("{}/login", base_url))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = response.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}