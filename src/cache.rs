use redis::AsyncCommands;
use redis::Client;

pub fn create_client(redis_url: &str) -> Result<Client, redis::RedisError> {
    Client::open(redis_url)
}

pub async fn get(client: &Client, key: &str) -> Option<String> {
    let mut conn = client.get_multiplexed_async_connection().await.ok()?;
    conn.get(key).await.ok()
}

pub async fn set(client: &Client, key: &str, value: &str, ttl_seconds: u64) -> bool {
    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return false,
    };
    let result: Result<(), _> = conn.set_ex(key, value, ttl_seconds).await;
    result.is_ok()
}

pub async fn delete(client: &Client, key: &str) -> bool {
    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return false,
    };
    let result: Result<(), _> = conn.del(key).await;
    result.is_ok()
}