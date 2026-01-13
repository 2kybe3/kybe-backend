FROM rust:latest AS builder
WORKDIR /usr/src/kybe-backend

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY migrations ./migrations
COPY .sqlx ./.sqlx

RUN cargo build --release && cp target/release/kybe-backend /usr/src/kybe-backend/kybe-backend

FROM debian:trixie-slim
WORKDIR /opt/backend

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/kybe-backend/kybe-backend /usr/bin/

EXPOSE 3000
CMD ["kybe-backend"]
