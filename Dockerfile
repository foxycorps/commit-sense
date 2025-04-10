# --- Builder Stage ---
FROM rust:1.86 as builder
WORKDIR /usr/src/commitsense
COPY . .
# Build release binary
RUN cargo build --release --locked


# --- Final Stage ---
FROM debian:bookworm-slim

# Install runtime dependencies
# - ca-certificates: For HTTPS
# - libssl3: Required for Rust TLS (newer version)
# - git: *** Now required as we call the git executable ***
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates \
       libssl3 \
       git \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /usr/src/commitsense/target/release/commit-sense /usr/local/bin/commit-sense

# Create a wrapper script to check for API key and configure Git
RUN echo '#!/bin/bash\n\
if [ -z "$OPENAI_API_KEY" ]; then\n\
  echo "Error: OPENAI_API_KEY environment variable is required"\n\
  echo "Run with: docker run -e OPENAI_API_KEY=your_api_key -t commitsense"\n\
  exit 1\n\
fi\n\
# Configure Git to trust the GitHub workspace directory\n\
if [ -d "/github/workspace" ]; then\n\
  git config --global --add safe.directory /github/workspace\n\
  echo "Configured Git to trust /github/workspace"\n\
fi\n\
# If first arg is -c, assume we are being run through GitHub Actions\n\
if [ "$1" = "-c" ]; then\n\
  # Pass all arguments to sh\n\
  exec sh "$@"\n\
else\n\
  # Otherwise, run commit-sense directly\n\
  exec commit-sense "$@"\n\
fi' > /usr/local/bin/docker-entrypoint.sh \
    && chmod +x /usr/local/bin/docker-entrypoint.sh

# Set the entrypoint to our wrapper script
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]