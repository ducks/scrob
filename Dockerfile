# Build stage
# Using nightly temporarily due to edition2024 in some dependencies (home, base64ct)
# TODO: Switch back to stable rust:1.84+ when edition2024 is stabilized
FROM rustlang/rust:nightly as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source and migrations
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build release binary (sqlx will run queries offline mode using cache if present)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/scrob /app/scrob

# Copy migrations and scripts
COPY migrations ./migrations
COPY scripts ./scripts

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=scrob=info

EXPOSE 3000

CMD ["/app/scrob"]
