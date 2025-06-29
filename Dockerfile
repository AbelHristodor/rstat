# =============================================================================
# STAGE 1: Base Chef Stage (Always runs on current architecture)
# =============================================================================
# This stage sets up the build environment with Rust, Zig, and necessary tools
FROM --platform=$BUILDPLATFORM rust:alpine AS chef

WORKDIR /app

# Set pkg-config sysroot for cross-compilation
ENV PKG_CONFIG_SYSROOT_DIR=/

# Install system dependencies:
# - musl-dev: C library headers for musl
# - openssl & openssl-dev: SSL/TLS library and development headers
# - zig: Cross-compilation toolchain
RUN apk add musl-dev openssl openssl-dev zig

# Install Rust tools for efficient builds:
# - cargo-zigbuild: Enables cross-compilation with Zig
# - cargo-chef: Caches dependencies to speed up builds
RUN cargo install --locked cargo-zigbuild cargo-chef

# Add target architectures for cross-compilation:
# - x86_64-unknown-linux-musl: 64-bit x86 Linux with musl libc
# - aarch64-unknown-linux-musl: 64-bit ARM Linux with musl libc
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# =============================================================================
# STAGE 2: Dependency Planning Stage
# =============================================================================
FROM chef AS planner

# Copy the entire project source code
COPY . .

# Generate a recipe of all dependencies needed for the project
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# STAGE 3: Dependency Building Stage
# =============================================================================
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build all dependencies for both target architectures using cargo-chef
# This step caches dependencies to avoid rebuilding them on every build
RUN cargo chef cook --recipe-path recipe.json --release --zigbuild \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl

# =============================================================================
# STAGE 4: Application Building Stage
# =============================================================================
COPY . .

# Enable SQLx offline mode to avoid database connection during build
ENV SQLX_OFFLINE=true

# Build the application for both target architectures using cargo-zigbuild
RUN cargo zigbuild -r \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl && \
    # Create output directory for the built binaries
    mkdir /app/linux && \
    # Copy ARM64 binary with descriptive name
    cp target/aarch64-unknown-linux-musl/release/prog /app/linux/arm64 && \
    # Copy AMD64 binary with descriptive name
    cp target/x86_64-unknown-linux-musl/release/prog /app/linux/amd64

# =============================================================================
# STAGE 5: Runtime Stage
# =============================================================================
FROM alpine:latest AS runtime

WORKDIR /app

ARG TARGETPLATFORM

# Copy the appropriate binary for the target platform from the builder stage
# The binary is copied to /app/prog for consistent naming
COPY --from=builder /app/${TARGETPLATFORM} /app/prog

# Set the default command to run the application
CMD ["/app/prog"]