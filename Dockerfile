# Corpex Demo — Desktop GUI via noVNC
#
# Builds the native Rust/egui app inside the container and exposes
# it over the web via Xvfb + x11vnc + noVNC. Zero source modifications.
#
# Usage:
#   docker build -t corpex-demo .
#   docker run -p 8080:8080 -e CH_API_KEY=your_key corpex-demo

# ── Stage 1: Build the Rust binary ────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build release binary
RUN cargo build --release --bin corpex

# ── Stage 2: Runtime with noVNC ───────────────────────────────────────
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Install X11, VNC, noVNC, and EGL/OpenGL for egui
RUN apt-get update && apt-get install -y --no-install-recommends \
    xvfb \
    x11vnc \
    novnc \
    websockify \
    libgl1 \
    libegl1 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libfontconfig1 \
    fonts-dejavu-core \
    openbox \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /build/target/release/corpex /usr/local/bin/corpex

# Copy the startup script
COPY docker-entrypoint.sh /entrypoint.sh
RUN sed -i 's/\r$//' /entrypoint.sh && chmod +x /entrypoint.sh

# Copy .env.example for reference
COPY .env.example /app/.env.example

WORKDIR /app

EXPOSE 8080

ENV DISPLAY=:99
ENV CH_API_KEY=""
ENV ANTHROPIC_API_KEY=""
ENV OPENAI_API_KEY=""

ENTRYPOINT ["/entrypoint.sh"]
