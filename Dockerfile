# ─── Build Stage ─────────────────────────────────────────────────────────────
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# DuckDB bundled feature compiles DuckDB from source — needs cmake + C++
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    g++ \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY crates/ ./crates/

RUN cargo build --package contextgenos-cli --release

# ─── Runtime Stage ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/contextgenos /usr/local/bin/contextgenos

# Context data lives in ~/.contextgenos
VOLUME ["/root/.contextgenos"]

ENTRYPOINT ["contextgenos"]
CMD ["--help"]
