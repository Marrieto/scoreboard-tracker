// routes/matches.rs — Match API handlers.
//
// Handles recording new matches, listing match history, editing matches,
// and deleting matches. The match recording endpoint creates a MatchRecord
// with a reverse-timestamp ID so matches are automatically sorted newest-first
// in Azure Table Storage.
//
// Authorization for editing:
//   - Admins can edit any match.
//   - Regular users can edit matches where their linked player_id matches
//     any of the 4 players in the match. This means only participants (or
//     admins) can correct scores.
//
// League filtering:
//   - The list endpoint accepts an optional `league_id` query parameter.
//   - When provided, only matches belonging to that league are returned.
//   - When omitted, all matches are returned (all-time view).

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;

use crate::auth::oidc::SessionClaims;
use crate::models::match_record::{CreateMatchRequest, MatchRecord, UpdateMatchRequest};
use crate::storage::client::StorageClient;
use crate::storage::matches::{self, MatchStorageError};

/// Map storage errors to HTTP responses.
///
/// The `Forbidden` variant maps to 403 — used when a user tries to edit
/// a match they don't have permission for.
impl IntoResponse for MatchStorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            MatchStorageError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            MatchStorageError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            MatchStorageError::Azure(_) => {
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

/// Query parameters for listing matches.
#[derive(Deserialize)]
pub struct ListMatchesQuery {
    /// Maximum number of matches to return.
    pub limit: Option<usize>,
    /// Filter to a specific league. Omit for all-time.
    pub league_id: Option<String>,
}

/// GET /api/matches — List recent matches.
///
/// Supports optional filtering by league_id. When a league_id is provided,
/// we filter in-memory after fetching from storage. This is fine for our
/// small dataset (<1000 matches). For larger datasets, you'd want an
/// Azure Table Storage filter or a secondary index.
pub async fn list_matches(
    State(storage): State<StorageClient>,
    Query(query): Query<ListMatchesQuery>,
) -> Result<Json<Vec<MatchRecord>>, MatchStorageError> {
    let mut all_matches = matches::list_matches(&storage, None).await?;

    // Filter by league if requested.
    if let Some(ref league_id) = query.league_id {
        all_matches.retain(|m| m.league_id.as_deref() == Some(league_id.as_str()));
    }

    // Apply limit after filtering (so we get `limit` matches from the filtered set).
    if let Some(limit) = query.limit {
        all_matches.truncate(limit);
    }

    Ok(Json(all_matches))
}

/// POST /api/matches — Record a new match result.
///
/// The `recorded_by` field is automatically set from the authenticated user's
/// session claims (their Microsoft OID), replacing the old "anonymous" hardcode.
pub async fn create_match(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Json(req): Json<CreateMatchRequest>,
) -> Result<(StatusCode, Json<MatchRecord>), MatchStorageError> {
    let played_at = req.played_at.unwrap_or_else(Utc::now);

    let record = MatchRecord::new(
        req.winner1_id,
        req.winner2_id,
        req.loser1_id,
        req.loser2_id,
        req.winner_score,
        req.loser_score,
        req.comment,
        claims.sub,
        played_at,
        req.league_id,
    );

    let created = matches::create_match(&storage, record).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/matches/{id} — Update an existing match.
///
/// Authorization: admin OR the user's player_id matches any of the 4 players
/// in the match. This ensures only participants (or admins) can fix scores.
///
/// Immutable fields preserved from the original: id, recorded_by, played_at.
/// Mutable fields from the request: players, scores, comment, league_id.
pub async fn update_match(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMatchRequest>,
) -> Result<Json<MatchRecord>, MatchStorageError> {
    // Fetch the existing match to check authorization and preserve immutable fields.
    let existing = matches::get_match(&storage, &id).await?;

    // Authorization check: admin or participant.
    let is_admin = claims.role == "admin";
    let is_participant = claims.player_id.as_ref().is_some_and(|pid| {
        // Check if the user's linked player_id matches any player in the match.
        // We check both the existing match players AND the new request players
        // to cover the case where the user is being removed from the match.
        pid == &existing.winner1_id
            || pid == &existing.winner2_id
            || pid == &existing.loser1_id
            || pid == &existing.loser2_id
    });

    if !is_admin && !is_participant {
        return Err(MatchStorageError::Forbidden(
            "Only admins or match participants can edit matches".to_string(),
        ));
    }

    // Build the updated record, preserving immutable fields from the original.
    let updated = MatchRecord {
        id: existing.id,
        winner1_id: req.winner1_id,
        winner2_id: req.winner2_id,
        loser1_id: req.loser1_id,
        loser2_id: req.loser2_id,
        winner_score: req.winner_score,
        loser_score: req.loser_score,
        comment: req.comment,
        recorded_by: existing.recorded_by,
        played_at: existing.played_at,
        league_id: req.league_id,
    };

    let result = matches::update_match(&storage, updated).await?;
    Ok(Json(result))
}

/// DELETE /api/matches/{id} — Delete a match.
///
/// Restricted to admins only. Regular users should edit matches instead
/// of deleting them (preserves audit trail).
pub async fn delete_match(
    State(storage): State<StorageClient>,
    Extension(claims): Extension<SessionClaims>,
    Path(id): Path<String>,
) -> Result<StatusCode, MatchStorageError> {
    // Only admins can delete matches.
    if claims.role != "admin" {
        return Err(MatchStorageError::Forbidden(
            "Only admins can delete matches".to_string(),
        ));
    }

    matches::delete_match(&storage, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
