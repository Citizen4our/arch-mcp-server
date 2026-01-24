FROM rust:1.85-slim AS builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml ./
RUN cargo generate-lockfile && cargo fetch

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false -m -d /app appuser

WORKDIR /app

COPY --from=builder /app/target/release/arch-mcp-server /app/arch-mcp-server

COPY example_docs/docs /app/docs

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8010

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8010/mcp || exit 1

CMD ["./arch-mcp-server", "--docs-root", "/app/docs/content", "--bind-address", "0.0.0.0:8010", "--rust-log", "info"]
