FROM rust:bookworm AS chef

RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin rstat-server

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR app
COPY --from=builder /app/target/release/rstat-server /usr/local/bin
COPY --from=builder /app/migrations /app/migrations

ENV MIGRATIONS_PATH=/app/migrations

ENTRYPOINT ["/usr/local/bin/rstat-server"]
CMD ["start"]