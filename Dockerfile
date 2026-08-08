FROM rust:1.96 AS builder

WORKDIR /build

# Cache dependencies by copying manifests before the full source tree.
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# The production image requires the CLI and HTTP thingd adapter features.
RUN cargo build --release --bin arqen --features cli,http-client

FROM debian:trixie-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/arqen /usr/local/bin/arqen

EXPOSE 8888

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fs http://127.0.0.1:8888/health || exit 1

ENTRYPOINT ["arqen"]
CMD ["start", "--host", "0.0.0.0", "--port", "8888", "--storage", "http"]
