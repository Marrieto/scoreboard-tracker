// auth/middleware.rs — Axum middleware for JWT session cookie validation.
//
// This middleware runs before protected route handlers. It:
//   1. Extracts the "session" cookie from the request
//   2. Validates the JWT signature and expiration
//   3. Injects the user's SessionClaims into request extensions
//
// Handlers can then access the authenticated user via:
//   `Extension(claims): Extension<SessionClaims>`
//
// Unprotected routes (like login, callback, and GET endpoints) skip this middleware.

use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::oidc::validate_session_token;
use crate::config::AppConfig;

/// The name of the cookie where we store the session JWT.
pub const SESSION_COOKIE_NAME: &str = "session";

/// Axum middleware that validates the session cookie.
///
/// If the cookie is missing or invalid, returns 401 Unauthorized.
/// If valid, injects `SessionClaims` into request extensions and continues.
pub async fn require_auth(
    request: Request,
    next: Next,
) -> Response {
    // Extract the AppConfig from request extensions (added by a layer).
    let config = request
        .extensions()
        .get::<AppConfig>()
        .cloned();

    let config = match config {
        Some(c) => c,
        None => {
            tracing::error!("AppConfig not found in request extensions");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Server configuration error"})),
            )
                .into_response();
        }
    };

    // Extract the session cookie from the Cookie header.
    let cookie_header = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok());

    let token = cookie_header.and_then(|cookies| {
        cookies
            .split(';')
            .map(|c| c.trim())
            .find(|c| c.starts_with(&format!("{SESSION_COOKIE_NAME}=")))
            .map(|c| c[SESSION_COOKIE_NAME.len() + 1..].to_string())
    });

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Not authenticated"})),
            )
                .into_response();
        }
    };

    // Validate the JWT.
    match validate_session_token(&config, &token) {
        Some(claims) => {
            // Inject claims into request extensions so handlers can access them.
            let mut request = request;
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        )
            .into_response(),
    }
}
