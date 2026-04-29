# TeachLink Development Environment
# Multi-stage Dockerfile for Soroban smart contract development

# Stage 1: Base image with Rust toolchain
FROM rust:1.77-slim as base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust toolchain
RUN rustup default stable && \
    rustup component add rustfmt clippy && \
    rustup target add wasm32-unknown-unknown

# Install Stellar CLI
RUN cargo install --locked stellar-cli --features opt

# Create app directory
WORKDIR /workspace

# Stage 2: Development environment with all tools
FROM base as development

# Set environment variables
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY contracts ./contracts
COPY scripts ./scripts
COPY config ./config
COPY docs ./docs

# Create .env from example if it doesn't exist
COPY .env.example .env.example

# Pre-build dependencies for faster rebuilds
RUN cargo fetch

# Default command
CMD ["/bin/bash"]

# Stage 3: Builder for production WASM
FROM base as builder

WORKDIR /workspace

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY contracts ./contracts

# Build optimized WASM
RUN cargo build --release --target wasm32-unknown-unknown

# Stage 4: Runtime with only built artifacts
FROM scratch as artifacts

# Copy built WASM files
COPY --from=builder /workspace/target/wasm32-unknown-unknown/release/*.wasm /

# Usage instructions:
#
# Development:
#   docker build --target development -t teachlink-dev .
#   docker run -it --rm -v $(pwd):/workspace teachlink-dev
#
# Build artifacts:
#   docker build --target builder -t teachlink-builder .
#   docker run --rm -v $(pwd)/target:/workspace/target teachlink-builder
#
# Extract WASM:
#   docker build --target artifacts --output target/wasm .
