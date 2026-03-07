FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src/kybe-backend

FROM chef AS planner

COPY Cargo.toml Cargo.lock ./

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/kybe-backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY src ./src
COPY .sqlx ./.sqlx
COPY assets ./assets
COPY migrations ./migrations
COPY Cargo.toml Cargo.lock ./
COPY static/pgp.txt ./static/pgp.txt

RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /opt/backend

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/kybe-backend/target/release/kybe-backend /usr/bin/
COPY static ./static

EXPOSE 3000
CMD ["kybe-backend"]
