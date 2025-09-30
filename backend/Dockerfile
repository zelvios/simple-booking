# Stage 1 — builder
FROM rust:bookworm AS builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev pkg-config libssl-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release || true

# Copy full source and build backend
COPY . .
RUN cargo build --release

# Install diesel_cli in builder
RUN cargo install diesel_cli --no-default-features --features postgres

# Stage 2 — runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 ca-certificates postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy migrations and diesel.toml for runtime migrations
COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/diesel.toml .

# Copy backend binary and diesel CLI
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# wait for Postgres, run migrations, then start backend
CMD ["sh", "-c", "until pg_isready -h db -p 5432; do sleep 1; done && diesel migration run --migration-dir ./migrations && exec /usr/local/bin/backend"]