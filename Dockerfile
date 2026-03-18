FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY .sqlx ./.sqlx

ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/workout-tracker .

CMD ["./workout-tracker"]