# =============================================================================
# STAGE 1: Base Chef Stage (Always runs on current architecture)
# =============================================================================
FROM --platform=$BUILDPLATFORM rust:alpine AS chef

WORKDIR /app

# Install system dependencies for static OpenSSL build
RUN apk add --no-cache musl-dev openssl openssl-dev \
    perl cmake make clang gcc libc-dev

# Install Rust tools for efficient builds:
RUN cargo install --locked cargo-zigbuild cargo-chef

RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Set environment variables for static OpenSSL
ENV OPENSSL_STATIC=1 \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include

# =============================================================================
# STAGE 2: Dependency Planning Stage
# =============================================================================
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# STAGE 3: Dependency Building Stage
# =============================================================================
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --recipe-path recipe.json --release --zigbuild \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl

# =============================================================================
# STAGE 4: Application Building Stage
# =============================================================================
COPY . .

ENV SQLX_OFFLINE=true

# Build the application for both target architectures using cargo-zigbuild
RUN cargo zigbuild -r \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl && \
    mkdir /app/linux && \
    cp target/aarch64-unknown-linux-musl/release/rstat-server /app/linux/arm64 && \
    cp target/x86_64-unknown-linux-musl/release/rstat-server /app/linux/amd64

# =============================================================================
# STAGE 5: Runtime Stage
# =============================================================================
FROM alpine:latest AS runtime

WORKDIR /app

ARG TARGETPLATFORM

# Helper script to select the correct binary based on TARGETPLATFORM
# This works for docker buildx (linux/amd64 or linux/arm64)
SHELL ["/bin/sh", "-c"]
RUN apk add --no-cache libgcc

COPY --from=builder /app/linux/amd64 /app/rstat-server-amd64
COPY --from=builder /app/linux/arm64 /app/rstat-server-arm64

RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
      cp /app/rstat-server-amd64 /app/rstat-server; \
    elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
      cp /app/rstat-server-arm64 /app/rstat-server; \
    else \
      echo "Unsupported TARGETPLATFORM: $TARGETPLATFORM" && exit 1; \
    fi

CMD ["/app/rstat-server"]