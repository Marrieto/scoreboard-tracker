// main.rs — Entry point for the Scoreboard Tracker backend.
//
// This server uses Axum to serve both:
//   1. A JSON REST API under `/api/*`
//   2. Pre-built SvelteKit static files for the frontend SPA
//
// The app reads configuration from environment variables (see config.rs),
// connects to Azure Table Storage for persistence, and uses Microsoft Entra ID
// (Azure AD) for user authentication via OIDC.
//
// Architecture:
//   Browser → Axum HTTP server → Azure Table Storage
//
// In development:
//   - Run the Rust backend: `cargo run` (serves API on port 3000)
//   - Run SvelteKit dev server: `cd frontend && npm run dev` (proxies /api to :3000)
//
// In production:
//   - The Docker image builds the frontend into static files and embeds them.
//   - Axum serves both the API and the static files from a single binary.

mod auth;
mod config;
mod models;
mod routes;
mod storage;

use axum::Router;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::storage::client::StorageClient;

#[tokio::main]
async fn main() {
    // ── Initialize tracing (structured logging) ─────────────────────────
    // The `RUST_LOG` env var controls log levels, e.g.:
    //   RUST_LOG=scoreboard=debug,tower_http=debug
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scoreboard=info,tower_http=info".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Load configuration from environment ─────────────────────────────
    // In development, use a `.env` file (loaded by dotenvy).
    // In production (Docker), env vars are set directly.
    dotenvy::dotenv().ok(); // Silently ignore if no .env file exists.
    let config = AppConfig::from_env();
    let port = config.port;

    // ── Initialize Azure Table Storage client ───────────────────────────
    let storage = StorageClient::new(&config);

    // Ensure our tables exist (creates them on first run).
    if let Err(e) = storage.ensure_tables_exist().await {
        tracing::error!("Failed to ensure Azure tables exist: {e}");
        tracing::warn!("Continuing anyway — tables may already exist or Azure may be unreachable");
    }

    // ── Build the application router ────────────────────────────────────
    //
    // The router is layered:
    //   /api/*  → JSON REST API (handled by our route handlers)
    //   /*      → Static files (SvelteKit build output)
    //
    // The `ServeDir` fallback serves the SPA's index.html for all unmatched
    // routes, so client-side routing works correctly.
    let app = Router::new()
        .nest("/api", routes::api_router(storage, config))
        .fallback_service(
            ServeDir::new("static").fallback(ServeFile::new("static/index.html")),
        )
        .layer(TraceLayer::new_for_http());

    // ── Start the server ────────────────────────────────────────────────
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Scoreboard server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
