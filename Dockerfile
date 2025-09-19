FROM rust:latest as builder

# Install system dependencies for building
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && \
    apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY src-tauri/Cargo.toml ./

# Copy source code
COPY src-tauri/src ./src/

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && \
    apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 opcodeuser

# Copy binary from builder
COPY --from=builder /app/target/release/opcode /usr/local/bin/opcode

# Create working directory
WORKDIR /app
RUN chown opcodeuser:opcodeuser /app

# Switch to non-root user
USER opcodeuser

# Expose API port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/api/health || exit 1

# Start API server
CMD ["opcode", "api", "--port", "3001"]