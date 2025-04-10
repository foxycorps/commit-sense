# --- Builder Stage ---
FROM rust:1.86 as builder
WORKDIR /usr/src/commitsense
COPY . .
# Build release binary
RUN cargo build --release --locked


# --- Final Stage ---
FROM debian:bullseye-slim

# Install runtime dependencies
# - ca-certificates: For HTTPS
# - libssl-dev: Common requirement for Rust TLS crates
# - git: *** Now required as we call the git executable ***
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates \
       libssl-dev \
       git \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /usr/src/commitsense/target/release/commit-sense /usr/local/bin/commitsense

# Set the entrypoint
ENTRYPOINT ["commitsense"]