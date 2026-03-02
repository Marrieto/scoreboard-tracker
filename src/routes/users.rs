// routes/users.rs — User management API handlers.
//
// These endpoints allow admins to manage users and let users link their
// accounts to player profiles. All endpoints require authentication
// (enforced by the auth middleware layer on the router).
//
// Endpoints:
//   GET  /api/users          — List all users (admin only)
//   PUT  /api/users/{oid}/role   — Change a user's role (admin only)
//   PUT  /api/users/{oid}/player — Link/unlink player profile (admin or self)
//
// Authorization is checked inside each handler using the `SessionClaims`
// injected by the auth middleware. This is a common Axum pattern — the
// middleware handles authentication (who are you?), and the handler handles
// authorization (what can you do?).

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::auth::oidc::SessionClaims;
use crate::models::user::{LinkPlayerRequest, UpdateUserRoleRequest};
use crate::storage::client::StorageClient;
use crate::storage::users::{self, UserStorageError};

/// Map UserStorageError variants to HTTP status codes.
///
/// Implementing `IntoResponse` for our error type lets Axum automatically
/// convert errors into HTTP responses. This is part of Axum's "return Result
/// from handlers" ergonomics — when a handler returns `Result<T, E>` where
/// both T and E implement `IntoResponse`, Axum handles the conversion.
impl IntoResponse for UserStorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            UserStorageError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            UserStorageError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            UserStorageError::Azure(_) => {
                // Log the actual Azure error but don't expose internals to clients.
                tracing::error!("Azure storage error: {self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// GET /api/users — List all users.
///
/// Admin-only endpoint. Regular users get a 403 Forbidden response.
///
/// The `Extension(claims)` extractor pulls the `SessionClaims` that was
/// injected into request extensions by the auth middleware. This is how
/// we access the authenticated user's identity in handlers.
pub async fn list_users(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
) -> Result<Json<Vec<crate::models::user::User>>, UserStorageError> {
    // Authorization check: only admins can list users.
    if claims.role != "admin" {
        return Err(UserStorageError::Forbidden(
            "Only admins can list users".to_string(),
        ));
    }

    let users = users::list_users(&storage).await?;
    Ok(Json(users))
}

/// PUT /api/users/{oid}/role — Change a user's role.
///
/// Admin-only endpoint. Accepts a JSON body with `{ "role": "admin" | "user" }`.
///
/// The `Path(oid)` extractor pulls the `{oid}` path parameter from the URL.
/// Axum matches the parameter name in the route definition to the extractor.
pub async fn update_user_role(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(oid): Path<String>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> Result<Json<crate::models::user::User>, UserStorageError> {
    // Authorization check: only admins can change roles.
    if claims.role != "admin" {
        return Err(UserStorageError::Forbidden(
            "Only admins can change user roles".to_string(),
        ));
    }

    let updated = users::update_user_role(&storage, &oid, &req.role).await?;
    Ok(Json(updated))
}

/// PUT /api/users/{oid}/player — Link or unlink a player profile.
///
/// Admins can link any user to any player. Regular users can only link themselves.
/// This supports the "claim your profile" workflow where users self-serve.
pub async fn link_player(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(oid): Path<String>,
    Json(req): Json<LinkPlayerRequest>,
) -> Result<Json<crate::models::user::User>, UserStorageError> {
    // Authorization check: admin can link anyone, users can only link themselves.
    if claims.role != "admin" && claims.sub != oid {
        return Err(UserStorageError::Forbidden(
            "You can only link your own player profile".to_string(),
        ));
    }

    let updated = users::link_player(&storage, &oid, req.player_id).await?;
    Ok(Json(updated))
}
