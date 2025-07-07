# ---- Stage 1: Build with Bun ----
FROM oven/bun:1.1 AS builder

WORKDIR /app

# Copy dependencies
COPY bun.lock package.json ./

# Install dependencies
RUN bun install

# Copy the rest of the app
COPY . .

# Build the Next.js app
RUN bun run build

# ---- Stage 2: Run with Bun ----
FROM oven/bun:1.1-slim

WORKDIR /app

# Copy built app and deps from builder
COPY --from=builder /app ./

# Expose the app port
EXPOSE 3000

# Start the Next.js production server
CMD ["bun", "start"]
