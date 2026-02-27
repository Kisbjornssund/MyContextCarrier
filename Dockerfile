# ─── Build Stage ─────────────────────────────────────────────────────────────
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Force git protocol — cargo sparse registry (HTTP/2) fails in Docker BuildKit network namespace
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git

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

# Context data lives in ~/.local/share/ContextGenOS (Linux data_local_dir)
VOLUME ["/root/.local/share/ContextGenOS"]

ENTRYPOINT ["contextgenos"]
CMD ["--help"]
