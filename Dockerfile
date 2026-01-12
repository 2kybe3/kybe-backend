FROM rust:latest AS builder
WORKDIR /usr/src/kybe-backend

ARG GIT_SHA
ENV GIT_SHA=${GIT_SHA}

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() { println!("hello"); }' > src/main.rs

RUN cargo build --release

COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY build.rs ./

RUN cargo build --release && cp target/release/kybe-backend /usr/src/kybe-backend/kybe-backend

FROM debian:trixie-slim
WORKDIR /opt/backend

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/kybe-backend/kybe-backend /usr/bin/

EXPOSE 3000
CMD ["kybe-backend"]
