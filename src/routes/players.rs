// routes/players.rs — Player API handlers.
//
// Each handler is an async function that takes Axum extractors (State, Path, Json)
// and returns an HTTP response. Axum automatically serializes the response body
// to JSON when we return `Json<T>`.
//
// Error handling: We map storage errors to appropriate HTTP status codes using
// `IntoResponse` implementations.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::models::player::{CreatePlayerRequest, Player, UpdatePlayerRequest};
use crate::storage::client::StorageClient;
use crate::storage::players::{self, PlayerStorageError};

/// Map storage errors to HTTP responses.
impl IntoResponse for PlayerStorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            PlayerStorageError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PlayerStorageError::AlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            PlayerStorageError::Azure(_) => {
                // Log the actual error but don't expose Azure internals to the client.
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

/// GET /api/players — List all players.
pub async fn list_players(
    State(storage): State<StorageClient>,
) -> Result<Json<Vec<Player>>, PlayerStorageError> {
    let players = players::list_players(&storage).await?;
    Ok(Json(players))
}

/// POST /api/players — Create a new player.
pub async fn create_player(
    State(storage): State<StorageClient>,
    Json(req): Json<CreatePlayerRequest>,
) -> Result<(StatusCode, Json<Player>), PlayerStorageError> {
    let player = Player {
        id: req.id,
        name: req.name,
        nickname: req.nickname,
        avatar_emoji: req.avatar_emoji,
    };

    let created = players::create_player(&storage, player).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/players/:id — Update an existing player.
pub async fn update_player(
    State(storage): State<StorageClient>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePlayerRequest>,
) -> Result<Json<Player>, PlayerStorageError> {
    let updated =
        players::update_player(&storage, &id, req.name, req.nickname, req.avatar_emoji).await?;
    Ok(Json(updated))
}

/// DELETE /api/players/:id — Delete a player.
pub async fn delete_player(
    State(storage): State<StorageClient>,
    Path(id): Path<String>,
) -> Result<StatusCode, PlayerStorageError> {
    players::delete_player(&storage, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
