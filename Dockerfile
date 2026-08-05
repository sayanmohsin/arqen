FROM rust:1.96 AS builder

WORKDIR /build

# Cache dependencies by copying manifests before the full source tree.
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# The `arqen` binary requires the `cli` feature.
RUN cargo build --release --bin arqen --features cli

FROM debian:trixie-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/arqen /usr/local/bin/arqen

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fs http://127.0.0.1:3000/health || exit 1

ENTRYPOINT ["arqen"]
CMD ["start", "--host", "0.0.0.0", "--port", "3000", "--storage", "memory"]
