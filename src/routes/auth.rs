// routes/auth.rs — Authentication route handlers.
//
// These endpoints handle the OIDC login flow:
//   GET  /api/auth/login    → Redirect to Microsoft login
//   GET  /api/auth/callback → Handle the redirect back from Microsoft
//   GET  /api/auth/me       → Return current user info (from session cookie)
//   POST /api/auth/logout   → Clear the session cookie
//
// The callback handler is the most complex — after validating the Microsoft
// ID token, it upserts the user record in Azure Table Storage and creates
// a session JWT that includes the user's role and linked player ID.
//
// The "first user is admin" logic is also handled here: if the users table
// is empty when someone logs in, they automatically get the admin role.

use axum::{
    Extension, Json,
    extract::Query,
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use serde::Deserialize;

use crate::auth::middleware::SESSION_COOKIE_NAME;
use crate::auth::oidc::{
    SessionClaims, authorize_url, create_session_token, decode_id_token_claims, exchange_code,
};
use crate::config::AppConfig;
use crate::models::user::User;
use crate::storage::client::StorageClient;
use crate::storage::users;

/// GET /api/auth/login — Redirect to Microsoft's login page.
pub async fn login(
    Extension(config): Extension<AppConfig>,
) -> Redirect {
    let url = authorize_url(&config);
    Redirect::temporary(&url)
}

/// Query parameters on the callback URL from Microsoft.
#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// GET /api/auth/callback — Handle the redirect back from Microsoft.
///
/// This is the heart of the login flow. After Microsoft authenticates the user
/// and redirects back here with an authorization code, we:
///   1. Exchange the code for tokens (ID token + access token).
///   2. Decode the ID token to get the user's identity (OID, name, email).
///   3. Upsert the user record in Azure Table Storage.
///   4. If this is the first user ever, assign them the "admin" role.
///   5. Create a signed session JWT cookie with role and player_id embedded.
///   6. Redirect to the home page.
///
/// The `Extension(storage)` extractor provides the StorageClient. Auth routes
/// need it as an Extension (not State) because they're on a different router
/// branch from the data routes. See routes/mod.rs for how this is wired up.
pub async fn callback(
    Extension(config): Extension<AppConfig>,
    Extension(storage): Extension<StorageClient>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    // Check for errors from Microsoft.
    if let Some(error) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown error");
        tracing::error!("OIDC error: {error} — {desc}");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Authentication failed",
                "detail": desc,
            })),
        )
            .into_response();
    }

    let code = match &query.code {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Missing authorization code"})),
            )
                .into_response();
        }
    };

    // Exchange the authorization code for tokens.
    let token_response = match exchange_code(&config, code).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token exchange failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token exchange failed"})),
            )
                .into_response();
        }
    };

    // Decode the ID token to get user info.
    let id_token = match &token_response.id_token {
        Some(t) => t,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "No ID token in response"})),
            )
                .into_response();
        }
    };

    let ms_claims = match decode_id_token_claims(id_token) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to decode ID token"})),
            )
                .into_response();
        }
    };

    // Extract user identity from Microsoft claims.
    let user_id = ms_claims
        .oid
        .or(ms_claims.sub)
        .unwrap_or_else(|| "unknown".to_string());
    let name = ms_claims.name.unwrap_or_else(|| "Unknown User".to_string());
    let email = ms_claims
        .preferred_username
        .unwrap_or_else(|| "unknown@unknown.com".to_string());

    // ── Upsert user in the users table ───────────────────────────────────
    //
    // On every login, we create or update the user record. This ensures:
    //   - New users are registered automatically (no separate signup flow).
    //   - Name/email changes in Azure AD are reflected in our system.
    //   - The "first user is admin" logic works on first-ever login.
    //
    // We first try to get the existing user to preserve their role and player_id.
    // If they don't exist, we determine their role (admin if first user, else "user").

    let (role, player_id) = match users::get_user(&storage, &user_id).await {
        Ok(existing) => {
            // User exists — preserve their current role and player_id,
            // but update their name/email in case it changed in Azure AD.
            let updated_user = User {
                oid: user_id.clone(),
                name: name.clone(),
                email: email.clone(),
                role: existing.role.clone(),
                player_id: existing.player_id.clone(),
                created_at: existing.created_at,
            };
            if let Err(e) = users::upsert_user(&storage, updated_user).await {
                tracing::warn!("Failed to update user on login: {e}");
            }
            (existing.role, existing.player_id)
        }
        Err(_) => {
            // User doesn't exist yet — create them.
            // Check if this is the very first user (they become admin).
            let role = match users::count_users(&storage).await {
                Ok(0) => {
                    tracing::info!("First user '{}' — assigning admin role", &name);
                    "admin".to_string()
                }
                Ok(_) => "user".to_string(),
                Err(e) => {
                    // If we can't count users, default to "user" (safe fallback).
                    tracing::warn!("Failed to count users: {e} — defaulting to 'user' role");
                    "user".to_string()
                }
            };

            let new_user = User {
                oid: user_id.clone(),
                name: name.clone(),
                email: email.clone(),
                role: role.clone(),
                player_id: None,
                created_at: Utc::now(),
            };

            if let Err(e) = users::upsert_user(&storage, new_user).await {
                tracing::error!("Failed to create user on login: {e}");
                // Continue anyway — the user can still use the app, they just
                // won't have a persistent record until next login.
            }

            (role, None)
        }
    };

    // Create our own session JWT with role and player_id embedded.
    let session_token =
        match create_session_token(&config, &user_id, &name, &email, &role, player_id) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to create session token: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create session"})),
                )
                    .into_response();
            }
        };

    // Set the session cookie and redirect to the home page.
    // HttpOnly: prevents JavaScript from reading the cookie (XSS protection).
    // SameSite=Lax: cookie sent on top-level navigations (needed for OIDC redirect).
    // Secure: only sent over HTTPS (omitted in development).
    let cookie_value = format!(
        "{SESSION_COOKIE_NAME}={session_token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400"
    );

    (
        StatusCode::SEE_OTHER,
        [
            (header::SET_COOKIE, cookie_value),
            (header::LOCATION, "/".to_string()),
        ],
    )
        .into_response()
}

/// GET /api/auth/me — Return the current user's info.
///
/// This endpoint is called by the frontend to check if the user is logged in
/// and to display their name/email. Now also returns role and player_id so
/// the frontend can show/hide admin features and determine edit permissions.
pub async fn me(
    claims: Option<Extension<SessionClaims>>,
) -> Response {
    match claims {
        Some(Extension(claims)) => Json(serde_json::json!({
            "authenticated": true,
            "user_id": claims.sub,
            "name": claims.name,
            "email": claims.email,
            "role": claims.role,
            "player_id": claims.player_id,
        }))
        .into_response(),
        None => Json(serde_json::json!({
            "authenticated": false,
        }))
        .into_response(),
    }
}

/// POST /api/auth/logout — Clear the session cookie.
pub async fn logout() -> Response {
    // Set the cookie with an expired Max-Age to delete it.
    let cookie_value =
        format!("{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_value)],
        Json(serde_json::json!({"message": "Logged out"})),
    )
        .into_response()
}
