// routes/leagues.rs — League management API handlers.
//
// Endpoints:
//   GET  /api/leagues          — List all leagues (any authenticated user)
//   GET  /api/leagues/{id}     — Get a single league (any authenticated user)
//   POST /api/leagues          — Create a new league (any authenticated user)
//   PUT  /api/leagues/{id}     — Update name/desc (admin or creator only)
//   POST /api/leagues/{id}/close — Close a league (admin or creator only)
//
// Any authenticated user can create a league — this encourages organic
// creation of seasons and tournaments. Only the creator or an admin can
// edit or close a league, preventing others from modifying seasons they
// didn't start.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;

use crate::auth::oidc::SessionClaims;
use crate::models::league::{CreateLeagueRequest, League, UpdateLeagueRequest};
use crate::storage::client::StorageClient;
use crate::storage::leagues::{self, LeagueStorageError};

/// Map LeagueStorageError variants to HTTP status codes.
///
/// Same pattern as PlayerStorageError and MatchStorageError — implement
/// IntoResponse so Axum can automatically convert errors in Result-returning
/// handlers.
impl IntoResponse for LeagueStorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            LeagueStorageError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            LeagueStorageError::AlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            LeagueStorageError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            LeagueStorageError::Azure(_) => {
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

/// GET /api/leagues — List all leagues.
///
/// Available to any authenticated user. Returns all leagues (active and closed).
pub async fn list_leagues(
    State(storage): State<StorageClient>,
) -> Result<Json<Vec<League>>, LeagueStorageError> {
    let leagues = leagues::list_leagues(&storage).await?;
    Ok(Json(leagues))
}

/// GET /api/leagues/{id} — Get a single league by ID.
pub async fn get_league(
    State(storage): State<StorageClient>,
    Path(id): Path<String>,
) -> Result<Json<League>, LeagueStorageError> {
    let league = leagues::get_league(&storage, &id).await?;
    Ok(Json(league))
}

/// POST /api/leagues — Create a new league.
///
/// Any authenticated user can create a league. The `created_by` field is
/// automatically set from the session claims to prevent spoofing.
pub async fn create_league(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Json(req): Json<CreateLeagueRequest>,
) -> Result<(StatusCode, Json<League>), LeagueStorageError> {
    let league = League {
        id: req.id,
        name: req.name,
        description: req.description,
        created_by: claims.sub,
        status: "active".to_string(),
        created_at: Utc::now(),
        closed_at: None,
    };

    let created = leagues::create_league(&storage, league).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/leagues/{id} — Update a league's name or description.
///
/// Only the league creator or an admin can update. We fetch the league first
/// to check the `created_by` field against the current user.
pub async fn update_league(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateLeagueRequest>,
) -> Result<Json<League>, LeagueStorageError> {
    // Check authorization: must be admin or the league creator.
    let existing = leagues::get_league(&storage, &id).await?;
    if claims.role != "admin" && claims.sub != existing.created_by {
        return Err(LeagueStorageError::Forbidden(
            "Only the creator or an admin can update this league".to_string(),
        ));
    }

    let updated = leagues::update_league(&storage, &id, req.name, req.description).await?;
    Ok(Json(updated))
}

/// POST /api/leagues/{id}/close — Close a league.
///
/// Only the league creator or an admin can close. Sets status to "closed"
/// and records the current timestamp as `closed_at`.
pub async fn close_league(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(id): Path<String>,
) -> Result<Json<League>, LeagueStorageError> {
    // Check authorization: must be admin or the league creator.
    let existing = leagues::get_league(&storage, &id).await?;
    if claims.role != "admin" && claims.sub != existing.created_by {
        return Err(LeagueStorageError::Forbidden(
            "Only the creator or an admin can close this league".to_string(),
        ));
    }

    let closed = leagues::close_league(&storage, &id).await?;
    Ok(Json(closed))
}
