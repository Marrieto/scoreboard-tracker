// routes/matches.rs — Match API handlers.
//
// Handles recording new matches and listing match history.
// The match recording endpoint creates a MatchRecord with a reverse-timestamp ID
// so matches are automatically sorted newest-first in Azure Table Storage.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;

use crate::models::match_record::{CreateMatchRequest, MatchRecord};
use crate::storage::client::StorageClient;
use crate::storage::matches::{self, MatchStorageError};

/// Map storage errors to HTTP responses.
impl IntoResponse for MatchStorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            MatchStorageError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
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
}

/// GET /api/matches — List recent matches.
pub async fn list_matches(
    State(storage): State<StorageClient>,
    Query(query): Query<ListMatchesQuery>,
) -> Result<Json<Vec<MatchRecord>>, MatchStorageError> {
    let matches = matches::list_matches(&storage, query.limit).await?;
    Ok(Json(matches))
}

/// POST /api/matches — Record a new match result.
pub async fn create_match(
    State(storage): State<StorageClient>,
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
        // TODO: Replace with authenticated user ID once auth is implemented (Step 4).
        "anonymous".to_string(),
        played_at,
    );

    let created = matches::create_match(&storage, record).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// DELETE /api/matches/:id — Delete a match (for corrections).
pub async fn delete_match(
    State(storage): State<StorageClient>,
    Path(id): Path<String>,
) -> Result<StatusCode, MatchStorageError> {
    matches::delete_match(&storage, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
