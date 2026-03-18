pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub jwt_secret: String,
    pub redis_url: String,
    pub rabbitmq_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("PORT must be a number");

        let host = std::env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let rabbitmq_url = std::env::var("RABBITMQ_URL")
            .expect("RABBITMQ_URL must be set");

        Config { database_url, port, host, jwt_secret, redis_url, rabbitmq_url }
    }
}