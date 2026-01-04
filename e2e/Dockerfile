# E2E Testing Dockerfile for Centy Daemon
# Multi-stage build: compiles daemon and sets up Node.js testing environment
#
# NOTE: This Dockerfile expects the build context to be the project root (parent dir)
# Run from e2e folder: docker build -f Dockerfile -t centy-e2e ..

# =============================================================================
# Stage 1: Build the Rust daemon
# =============================================================================
FROM rustlang/rust:nightly-alpine AS rust-builder

RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Remove the dummy build artifacts
RUN rm -rf src target/release/.fingerprint/centy-daemon-* target/release/deps/centy_daemon-*

# Copy actual source files
COPY src ./src
COPY proto ./proto
COPY build.rs ./

# Build the actual daemon
RUN cargo build --release

# =============================================================================
# Stage 2: Node.js E2E test runner
# =============================================================================
FROM node:22-alpine AS e2e-runner

# Install required system dependencies
RUN apk add --no-cache \
    bash \
    git \
    ca-certificates \
    protobuf

# Install pnpm
RUN corepack enable && corepack prepare pnpm@10.24.0 --activate

WORKDIR /app

# Copy the compiled daemon from the rust-builder stage
COPY --from=rust-builder /app/target/release/centy-daemon /usr/local/bin/centy-daemon

# Copy proto files (needed for gRPC client)
COPY proto ./proto

# Copy e2e test files
COPY e2e ./e2e

WORKDIR /app/e2e

# Install Node.js dependencies
RUN pnpm install --frozen-lockfile

# Set environment variables
ENV CENTY_DAEMON_ADDR=127.0.0.1:50051
ENV NODE_ENV=test
ENV CI=true

# Create test output directory for snapshots
RUN mkdir -p /app/e2e/test-output/snapshots

# Default command: start daemon and run tests
CMD ["sh", "-c", "centy-daemon & sleep 2 && pnpm test"]
