FROM rust:latest AS builder

WORKDIR /usr/src/kybe-backend

COPY Cargo.toml Cargo.lock* ./

RUN mkdir -p src && echo 'fn main() { println!("hello"); }' > src/main.rs
RUN cargo fetch

COPY src ./src

RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/kybe-backend/target/release/kybe-backend .

EXPOSE 3000

CMD ["./kybe-backend"]