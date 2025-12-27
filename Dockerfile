FROM rust:latest AS builder
WORKDIR /usr/src/kybe-backend

COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo 'fn main() { println!("hello"); }' > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY src ./src
COPY migrations ./migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

FROM debian:trixie-slim
WORKDIR /opt/backend

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/kybe-backend/target/release/kybe-backend /usr/bin/

CMD ["kybe-backend"]