# Build stage
FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /app

# Copy workspace manifest and lock
COPY Cargo.toml Cargo.lock ./

# Copy daemon package
COPY daemon/Cargo.toml ./daemon/
COPY daemon/src ./daemon/src
COPY daemon/build.rs ./daemon/
COPY daemon/build ./daemon/build

# Copy proto definitions
COPY proto ./proto

# Copy xtask (workspace member — manifest only needed for resolution)
COPY xtask/Cargo.toml ./xtask/

# Build release binary
RUN cargo build --release -p centy-daemon

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/centy-daemon /app/centy-daemon

# Create data directory
RUN mkdir -p /data

ENV CENTY_DAEMON_ADDR=0.0.0.0:50051

EXPOSE 50051

CMD ["/app/centy-daemon"]
