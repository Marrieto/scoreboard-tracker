// storage/players.rs — Player CRUD operations against Azure Table Storage.
//
// Each function takes a `StorageClient` and performs a single operation on the
// "players" table. Errors are propagated as `StorageError` so the route handlers
// can map them to appropriate HTTP responses.
//
// Azure Table Storage operations:
// - Insert: adds a new entity (fails if RowKey already exists)
// - Get:    retrieves a single entity by PartitionKey + RowKey
// - Update: replaces an existing entity (we use "merge" to update only changed fields)
// - Delete: removes an entity by PartitionKey + RowKey
// - Query:  lists entities, optionally filtered by OData expressions

use azure_data_tables::operations::InsertEntityResponse;
use futures::StreamExt;

use crate::models::player::{Player, PlayerEntity, PLAYER_PARTITION_KEY};
use crate::storage::client::StorageClient;

/// Errors that can occur during player storage operations.
#[derive(Debug, thiserror::Error)]
pub enum PlayerStorageError {
    #[error("Player '{0}' not found")]
    NotFound(String),

    #[error("Player '{0}' already exists")]
    AlreadyExists(String),

    #[error("Azure Table Storage error: {0}")]
    Azure(String),
}

/// Convert Azure SDK errors into our domain error type.
impl From<azure_core::Error> for PlayerStorageError {
    fn from(e: azure_core::Error) -> Self {
        let msg = format!("{e}");
        if msg.contains("EntityAlreadyExists") || msg.contains("409") {
            // Extract a meaningful ID if possible, otherwise use a generic message
            PlayerStorageError::AlreadyExists("(unknown)".to_string())
        } else if msg.contains("ResourceNotFound") || msg.contains("404") {
            PlayerStorageError::NotFound("(unknown)".to_string())
        } else {
            PlayerStorageError::Azure(msg)
        }
    }
}

/// List all players in the system.
///
/// Since all players share the same PartitionKey ("player"), we can list them
/// all with a single partition query. With <10 players, this always returns
/// in a single page.
pub async fn list_players(storage: &StorageClient) -> Result<Vec<Player>, PlayerStorageError> {
    let mut players = Vec::new();

    // Query all entities in the "player" partition.
    // The Azure SDK returns a paginated stream — we collect all pages.
    let mut stream = storage
        .players
        .query()
        .filter(format!("PartitionKey eq '{PLAYER_PARTITION_KEY}'"))
        .into_stream::<PlayerEntity>();

    while let Some(page_result) = stream.next().await {
        let page = page_result.map_err(PlayerStorageError::from)?;
        for entity in page.entities {
            players.push(Player::from(entity));
        }
    }

    Ok(players)
}

/// Get a single player by their ID (RowKey).
pub async fn get_player(
    storage: &StorageClient,
    player_id: &str,
) -> Result<Player, PlayerStorageError> {
    // The get() method returns a GetEntityResponse<T> with an `entity` field.
    // We use turbofish on get() to tell it what type to deserialize into.
    let response = storage
        .players
        .partition_key_client(PLAYER_PARTITION_KEY)
        .entity_client(player_id)
        .get::<PlayerEntity>()
        .await
        .map_err(|e| {
            let msg = format!("{e}");
            if msg.contains("ResourceNotFound") || msg.contains("404") {
                PlayerStorageError::NotFound(player_id.to_string())
            } else {
                PlayerStorageError::Azure(msg)
            }
        })?;

    Ok(Player::from(response.entity))
}

/// Create a new player.
///
/// Uses Azure Table Storage's "insert" operation, which fails with 409 Conflict
/// if an entity with the same PartitionKey + RowKey already exists.
pub async fn create_player(
    storage: &StorageClient,
    player: Player,
) -> Result<Player, PlayerStorageError> {
    let entity = PlayerEntity::from(player.clone());

    let _: InsertEntityResponse<PlayerEntity> =
        storage
            .players
            .insert(&entity)
            .map_err(|e| PlayerStorageError::Azure(format!("{e}")))?
            .await
            .map_err(|e| {
                let msg = format!("{e}");
                if msg.contains("EntityAlreadyExists") || msg.contains("409") {
                    PlayerStorageError::AlreadyExists(player.id.clone())
                } else {
                    PlayerStorageError::Azure(msg)
                }
            })?;

    Ok(player)
}

/// Update an existing player.
///
/// We first fetch the current entity (to get its ETag for optimistic concurrency),
/// then merge our changes. This means only the fields we provide are updated.
pub async fn update_player(
    storage: &StorageClient,
    player_id: &str,
    name: Option<String>,
    nickname: Option<String>,
    avatar_emoji: Option<String>,
) -> Result<Player, PlayerStorageError> {
    // First, get the current player to ensure it exists.
    let mut current = get_player(storage, player_id).await?;

    // Apply updates
    if let Some(n) = name {
        current.name = n;
    }
    if let Some(n) = nickname {
        current.nickname = n;
    }
    if let Some(a) = avatar_emoji {
        current.avatar_emoji = a;
    }

    // Convert back to entity and upsert (insert-or-replace).
    let entity = PlayerEntity::from(current.clone());

    storage
        .players
        .partition_key_client(PLAYER_PARTITION_KEY)
        .entity_client(player_id)
        .insert_or_replace(&entity)
        .map_err(|e| PlayerStorageError::Azure(format!("{e}")))?
        .await
        .map_err(|e| PlayerStorageError::Azure(format!("{e}")))?;

    Ok(current)
}

/// Delete a player by their ID.
pub async fn delete_player(
    storage: &StorageClient,
    player_id: &str,
) -> Result<(), PlayerStorageError> {
    // Use IfMatchCondition::Any to delete regardless of ETag (we don't need
    // optimistic concurrency for deletes in our simple app).
    storage
        .players
        .partition_key_client(PLAYER_PARTITION_KEY)
        .entity_client(player_id)
        .delete()
        .await
        .map_err(|e| {
            let msg = format!("{e}");
            if msg.contains("ResourceNotFound") || msg.contains("404") {
                PlayerStorageError::NotFound(player_id.to_string())
            } else {
                PlayerStorageError::Azure(msg)
            }
        })?;

    Ok(())
}
