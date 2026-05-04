# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cache dependencies by building a dummy binary first
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/loser_bot /usr/local/bin/loser_bot
CMD ["loser_bot"]
