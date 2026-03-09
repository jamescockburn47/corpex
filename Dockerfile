# ── Stage 1: Build WASM binary with Trunk ────────────────────────────
FROM rust:1.82-slim AS builder

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY index.html ./
COPY Trunk.toml ./

RUN trunk build --release

# ── Stage 2: Serve with nginx ────────────────────────────────────────
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
