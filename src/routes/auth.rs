// routes/auth.rs — Authentication route handlers.
//
// These endpoints handle the OIDC login flow:
//   GET  /api/auth/login    → Redirect to Microsoft login
//   GET  /api/auth/callback → Handle the redirect back from Microsoft
//   GET  /api/auth/me       → Return current user info (from session cookie)
//   POST /api/auth/logout   → Clear the session cookie

use axum::{
    Extension, Json,
    extract::Query,
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::auth::middleware::SESSION_COOKIE_NAME;
use crate::auth::oidc::{
    SessionClaims, authorize_url, create_session_token, decode_id_token_claims, exchange_code,
};
use crate::config::AppConfig;

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
/// Microsoft redirects here with either:
///   ?code=AUTHORIZATION_CODE (success)
///   ?error=ERROR&error_description=DESCRIPTION (failure)
pub async fn callback(
    Extension(config): Extension<AppConfig>,
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

    // Create our own session JWT.
    let user_id = ms_claims
        .oid
        .or(ms_claims.sub)
        .unwrap_or_else(|| "unknown".to_string());
    let name = ms_claims.name.unwrap_or_else(|| "Unknown User".to_string());
    let email = ms_claims
        .preferred_username
        .unwrap_or_else(|| "unknown@unknown.com".to_string());

    let session_token = match create_session_token(&config, &user_id, &name, &email) {
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
/// and to display their name/email.
pub async fn me(
    claims: Option<Extension<SessionClaims>>,
) -> Response {
    match claims {
        Some(Extension(claims)) => Json(serde_json::json!({
            "authenticated": true,
            "user_id": claims.sub,
            "name": claims.name,
            "email": claims.email,
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
