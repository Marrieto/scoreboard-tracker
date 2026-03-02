// routes/mod.rs — API route module and router composition.
//
// All API routes are mounted under `/api/` and composed into a single Axum Router.
// The router is then merged with static file serving in main.rs.
//
// Auth strategy:
//   - Auth endpoints (login, callback, logout) are always public.
//   - The `/api/auth/me` endpoint uses optional auth (returns info if logged in).
//   - All data endpoints (players, matches, leaderboard, users, leagues) require auth.
//   - Authorization (role checks) is handled inside individual handlers.
//
// Router structure:
//   - Auth routes get both AppConfig and StorageClient as Extensions (the callback
//     needs storage to upsert user records on login).
//   - Data routes get StorageClient as State (the standard Axum pattern for shared
//     state that handlers depend on).

pub mod auth;
pub mod leaderboard;
pub mod leagues;
pub mod matches;
pub mod players;
pub mod users;

use axum::{Extension, Router, middleware, routing::{delete, get, post, put}};

use crate::auth::middleware::require_auth;
use crate::config::AppConfig;
use crate::storage::client::StorageClient;

/// Build the API router with all endpoints.
///
/// The `AppConfig` is injected as an Extension (for auth middleware/handlers).
/// The `StorageClient` is injected both as Extension (for auth routes) and as
/// State (for data routes). This dual injection is needed because auth routes
/// and data routes are separate router branches with different middleware layers.
pub fn api_router(storage: StorageClient, config: AppConfig) -> Router {
    // Auth routes — always public (no auth middleware).
    // These need StorageClient as Extension because the callback upserts user records.
    let auth_routes = Router::new()
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .layer(Extension(storage.clone()));

    // Protected data routes — require authentication.
    let data_routes = Router::new()
        // Player endpoints
        .route("/players", get(players::list_players))
        .route("/players", post(players::create_player))
        .route("/players/{id}", put(players::update_player))
        .route("/players/{id}", delete(players::delete_player))
        // Match endpoints
        .route("/matches", get(matches::list_matches))
        .route("/matches", post(matches::create_match))
        .route("/matches/{id}", put(matches::update_match))
        .route("/matches/{id}", delete(matches::delete_match))
        // Leaderboard & stats endpoints
        .route("/leaderboard", get(leaderboard::get_leaderboard))
        .route("/players/{id}/stats", get(leaderboard::get_player_stats))
        .route("/rivalries", get(leaderboard::get_rivalries))
        // User management endpoints
        .route("/users", get(users::list_users))
        .route("/users/{oid}/role", put(users::update_user_role))
        .route("/users/{oid}/player", put(users::link_player))
        // League endpoints
        .route("/leagues", get(leagues::list_leagues))
        .route("/leagues", post(leagues::create_league))
        .route("/leagues/{id}", get(leagues::get_league))
        .route("/leagues/{id}", put(leagues::update_league))
        .route("/leagues/{id}/close", post(leagues::close_league))
        // Data handlers need the StorageClient as state.
        .with_state(storage)
        // Protect all data routes with auth middleware.
        .layer(middleware::from_fn(require_auth));

    // Combine auth and data routes, both sharing the AppConfig extension.
    Router::new()
        .merge(auth_routes)
        .merge(data_routes)
        .layer(Extension(config))
}
