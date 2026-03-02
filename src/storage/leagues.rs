// storage/leagues.rs — League CRUD operations against Azure Table Storage.
//
// Manages the "leagues" table. Leagues are stored with PartitionKey "league"
// and RowKey as the league's slug ID (e.g., "spring-2026").
//
// Operations follow the same patterns as players.rs and matches.rs:
//   - Paginated stream queries for listing
//   - Point reads for getting a single entity
//   - Insert for creation (fails on duplicate)
//   - Insert-or-replace for updates

use azure_data_tables::operations::InsertEntityResponse;
use futures::StreamExt;

use crate::models::league::{League, LeagueEntity, LEAGUE_PARTITION_KEY};
use crate::storage::client::StorageClient;

/// Errors that can occur during league storage operations.
#[derive(Debug, thiserror::Error)]
pub enum LeagueStorageError {
    /// The requested league was not found.
    #[error("League '{0}' not found")]
    NotFound(String),

    /// A league with this ID already exists (insert conflict).
    #[error("League '{0}' already exists")]
    AlreadyExists(String),

    /// The caller doesn't have permission for this operation.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// An unexpected error from the Azure SDK.
    #[error("Azure Table Storage error: {0}")]
    Azure(String),
}

/// Convert Azure SDK errors into our domain error type.
impl From<azure_core::Error> for LeagueStorageError {
    fn from(e: azure_core::Error) -> Self {
        let msg = format!("{e}");
        if msg.contains("EntityAlreadyExists") || msg.contains("409") {
            LeagueStorageError::AlreadyExists("(unknown)".to_string())
        } else if msg.contains("ResourceNotFound") || msg.contains("404") {
            LeagueStorageError::NotFound("(unknown)".to_string())
        } else {
            LeagueStorageError::Azure(msg)
        }
    }
}

/// List all leagues.
///
/// Returns all leagues from the "league" partition, ordered by Azure's
/// default ascending RowKey sort (alphabetical by slug ID).
pub async fn list_leagues(storage: &StorageClient) -> Result<Vec<League>, LeagueStorageError> {
    let mut leagues = Vec::new();

    let mut stream = storage
        .leagues
        .query()
        .filter(format!("PartitionKey eq '{LEAGUE_PARTITION_KEY}'"))
        .into_stream::<LeagueEntity>();

    while let Some(page_result) = stream.next().await {
        let page = page_result.map_err(LeagueStorageError::from)?;
        for entity in page.entities {
            match League::try_from(entity) {
                Ok(league) => leagues.push(league),
                Err(e) => {
                    tracing::warn!("Skipping league with invalid dates: {e}");
                }
            }
        }
    }

    Ok(leagues)
}

/// Get a single league by its ID (RowKey).
pub async fn get_league(
    storage: &StorageClient,
    league_id: &str,
) -> Result<League, LeagueStorageError> {
    let response = storage
        .leagues
        .partition_key_client(LEAGUE_PARTITION_KEY)
        .entity_client(league_id)
        .get::<LeagueEntity>()
        .await
        .map_err(|e| {
            let msg = format!("{e}");
            if msg.contains("ResourceNotFound") || msg.contains("404") {
                LeagueStorageError::NotFound(league_id.to_string())
            } else {
                LeagueStorageError::Azure(msg)
            }
        })?;

    League::try_from(response.entity)
        .map_err(|e| LeagueStorageError::Azure(format!("Failed to parse league: {e}")))
}

/// Create a new league.
///
/// Uses Azure Table Storage's "insert" operation which fails with 409 if
/// a league with the same ID already exists.
pub async fn create_league(
    storage: &StorageClient,
    league: League,
) -> Result<League, LeagueStorageError> {
    let entity = LeagueEntity::from(league.clone());

    let _: InsertEntityResponse<LeagueEntity> =
        storage
            .leagues
            .insert(&entity)
            .map_err(|e| LeagueStorageError::Azure(format!("{e}")))?
            .await
            .map_err(|e| {
                let msg = format!("{e}");
                if msg.contains("EntityAlreadyExists") || msg.contains("409") {
                    LeagueStorageError::AlreadyExists(league.id.clone())
                } else {
                    LeagueStorageError::Azure(msg)
                }
            })?;

    Ok(league)
}

/// Update an existing league (name, description).
///
/// Performs a read-modify-write: fetch current league, apply changes, upsert.
/// This preserves all fields not being updated.
pub async fn update_league(
    storage: &StorageClient,
    league_id: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<League, LeagueStorageError> {
    let mut current = get_league(storage, league_id).await?;

    if let Some(n) = name {
        current.name = n;
    }
    if let Some(d) = description {
        current.description = d;
    }

    let entity = LeagueEntity::from(current.clone());

    storage
        .leagues
        .partition_key_client(LEAGUE_PARTITION_KEY)
        .entity_client(league_id)
        .insert_or_replace(&entity)
        .map_err(|e| LeagueStorageError::Azure(format!("{e}")))?
        .await
        .map_err(|e| LeagueStorageError::Azure(format!("{e}")))?;

    Ok(current)
}

/// Close a league (set status to "closed" and record the closed_at timestamp).
///
/// Once closed, the league still appears in the list and its stats are still
/// viewable, but the UI should prevent assigning new matches to it.
pub async fn close_league(
    storage: &StorageClient,
    league_id: &str,
) -> Result<League, LeagueStorageError> {
    let mut current = get_league(storage, league_id).await?;

    current.status = "closed".to_string();
    current.closed_at = Some(chrono::Utc::now());

    let entity = LeagueEntity::from(current.clone());

    storage
        .leagues
        .partition_key_client(LEAGUE_PARTITION_KEY)
        .entity_client(league_id)
        .insert_or_replace(&entity)
        .map_err(|e| LeagueStorageError::Azure(format!("{e}")))?
        .await
        .map_err(|e| LeagueStorageError::Azure(format!("{e}")))?;

    Ok(current)
}
