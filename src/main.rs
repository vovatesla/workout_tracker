use axum::{
    routing::{get, post, put, delete},
    Router,
    response::Response,
    http::{header, HeaderValue},
};
use axum::middleware as axum_middleware;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use sqlx::PgPool;

pub mod models;
pub mod errors;
pub mod repository;
pub mod handlers;
pub mod config;
pub mod auth;
pub mod user_repository;
pub mod auth_handlers;
pub mod middleware;
pub mod cache;
pub mod queue;

use middleware::require_auth;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();
    
    let config = config::Config::from_env();

    let pool = PgPool::connect(&config.database_url).await.unwrap();

    let redis_client = cache::create_client(&config.redis_url)
        .expect("Failed to create Redis client");

    let rabbit_connection = queue::create_connection(&config.rabbitmq_url)
        .await
        .expect("Failed to connect to RabbitMQ");

    let rabbit_channel = queue::create_channel(&rabbit_connection)
        .await
        .expect("Failed to create RabbitMQ channel");

    queue::declare_queue(&rabbit_channel, "workout_created")
        .await
        .expect("Failed to declare queue");

    let worker_channel = queue::create_channel(&rabbit_connection)
        .await
        .expect("Failed to create worker channel");

    queue::declare_queue(&worker_channel, "workout_created")
        .await
        .expect("Failed to declare worker queue");

    tokio::spawn(async move {
        queue::start_worker(&worker_channel, "workout_created").await;
    });

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
        .layer(axum_middleware::map_response(add_charset))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    pool.close().await;
    println!("Server stopped");
}

async fn add_charset(mut response: Response) -> Response {
    if let Some(ct) = response.headers().get(header::CONTENT_TYPE) {
        if ct.as_bytes().starts_with(b"application/json") {
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json; charset=utf-8"),
            );
        }
    }
    response
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    println!("Shutting down...");
}