use lapin::{
    options::*,
    types::FieldTable,
    BasicProperties,
    Connection,
    ConnectionProperties,
    Channel,
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutCreatedEvent {
    pub workout_id: i64,
    pub user_id: i64,
    pub muscle_group: String,
}

pub async fn create_connection(url: &str) -> Result<Connection, lapin::Error> {
    Connection::connect(url, ConnectionProperties::default()).await
}

pub async fn create_channel(connection: &Connection) -> Result<Channel, lapin::Error> {
    connection.create_channel().await
}

pub async fn declare_queue(channel: &Channel, queue_name: &str) -> Result<(), lapin::Error> {
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn publish(channel: &Channel, queue_name: &str, event: &WorkoutCreatedEvent) -> Result<(), lapin::Error> {
    let payload = serde_json::to_vec(event).unwrap_or_default();

    channel
        .basic_publish(
            "", 
            queue_name,
            BasicPublishOptions::default(),
            &payload, 
            BasicProperties::default()
                .with_delivery_mode(2),
        )
        .await?
        .await?; 

    Ok(())
}

pub async fn start_worker(channel: &Channel, queue_name: &str) {
    use lapin::options::BasicConsumeOptions;
    use futures_util::StreamExt;

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "workout_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to start consumer");

    tracing::info!("Воркер запущен, ждём сообщений...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            match serde_json::from_slice::<WorkoutCreatedEvent>(&delivery.data) {
                Ok(event) => {
                    tracing::info!(
                        "Обработано событие: воркаут id={} юзер={} группа={}",
                        event.workout_id,
                        event.user_id,
                        event.muscle_group
                    );
                    delivery.ack(BasicAckOptions::default()).await.ok();
                }
                Err(e) => {
                    tracing::error!("Не удалось десериализовать сообщение: {}", e);
                }
            }
        }
    }
}