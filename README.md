# workout-tracker

A REST API for tracking workouts, built with Rust.

## Tech Stack

- **Rust** (edition 2021) + **Axum 0.7**
- **SQLx 0.8** — PostgreSQL with compile-time query checking
- **Redis** — workout list caching
- **RabbitMQ** — event queue
- **JWT** — authentication
- **Docker / Docker Compose**

## Getting Started

### 1. Environment variables

```bash
cp .env.example .env
```

Fill in `.env` with your values (see `.env.example`).

### 2. Run with Docker Compose

```bash
docker-compose up --build
```

### 3. Or run locally

```bash
# Start dependencies
docker-compose up db redis rabbitmq

# Apply migrations
sqlx migrate run

# Start server
cargo run
```

Server runs at `http://127.0.0.1:3000`.

## API

### Auth

| Method | Path      | Description         |
|--------|-----------|---------------------|
| POST   | /register | Register            |
| POST   | /login    | Login, receive JWT  |

Protected routes require:
```
Authorization: Bearer <token>
```

### Workouts

| Method | Path          | Description       |
|--------|---------------|-------------------|
| GET    | /health       | Health check      |
| GET    | /ping         | Ping              |
| GET    | /workouts     | List workouts     |
| POST   | /workouts     | Create workout    |
| GET    | /workouts/:id | Get workout       |
| PUT    | /workouts/:id | Update workout    |
| DELETE | /workouts/:id | Delete workout    |

## Development

```bash
cargo build
cargo test
SQLX_OFFLINE=true cargo build  # build without a live DB
```
