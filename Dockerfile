# Dockerfile — Multi-stage build for the Scoreboard Tracker.
#
# Stage 1: Build the SvelteKit frontend into static files.
# Stage 2: Build the Rust backend binary.
# Stage 3: Minimal runtime image with just the binary + static files.

# ── Stage 1: Frontend build ──────────────────────────────────────────
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Backend build ───────────────────────────────────────────
FROM rust:1-alpine AS backend-builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
# Copy the built frontend static files so they can be embedded or served
COPY --from=frontend-builder /app/frontend/build ./static
RUN cargo build --release

# ── Stage 3: Minimal runtime ─────────────────────────────────────────
FROM scratch
COPY --from=backend-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
WORKDIR /app
COPY --from=backend-builder /app/target/release/scoreboard ./
COPY --from=backend-builder /app/static ./static
EXPOSE 3000
CMD ["./scoreboard"]
