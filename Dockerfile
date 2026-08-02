FROM rust:1.96 as builder

WORKDIR /app

# Build from the `/ancatag` parent context so the Compose Dockerfile path
# remains stable. thingd is consumed from crates.io.
COPY arqen/Cargo.toml arqen/Cargo.lock ./arqen/
COPY arqen/crates/ ./arqen/crates/
COPY arqen/cli/ ./arqen/cli/
WORKDIR /app/arqen

# Build dependencies (cached)
RUN cargo build --release --bin arqen

# Copy application code (if any)
# For now, we just have the CLI

# Runtime stage
FROM debian:bookworm-slim as runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/arqen /usr/local/bin/arqen

EXPOSE 3000

ENTRYPOINT ["arqen"]
CMD ["start", "--host", "0.0.0.0", "--port", "3000", "--storage", "memory"]
