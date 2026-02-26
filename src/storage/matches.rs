// storage/matches.rs — Match CRUD operations against Azure Table Storage.
//
// Similar to players.rs but for match records. The key difference is the
// reverse-timestamp RowKey strategy that gives us newest-first ordering
// for free from Azure Table Storage's default ascending sort.

use futures::StreamExt;

use crate::models::match_record::{MatchEntity, MatchRecord, MATCH_PARTITION_KEY};
use crate::storage::client::StorageClient;

/// Errors that can occur during match storage operations.
#[derive(Debug, thiserror::Error)]
pub enum MatchStorageError {
    #[error("Match '{0}' not found")]
    NotFound(String),

    #[error("Azure Table Storage error: {0}")]
    Azure(String),
}

impl From<azure_core::Error> for MatchStorageError {
    fn from(e: azure_core::Error) -> Self {
        let msg = format!("{e}");
        if msg.contains("ResourceNotFound") || msg.contains("404") {
            MatchStorageError::NotFound("(unknown)".to_string())
        } else {
            MatchStorageError::Azure(msg)
        }
    }
}

/// List recent matches, optionally limited to `limit` results.
///
/// Because our RowKeys use a reverse timestamp, Azure Table Storage's default
/// ascending sort gives us newest matches first — no client-side sorting needed!
///
/// The `limit` parameter controls how many matches to return. Pass `None` to
/// get all matches (fine for our small dataset).
pub async fn list_matches(
    storage: &StorageClient,
    limit: Option<usize>,
) -> Result<Vec<MatchRecord>, MatchStorageError> {
    let mut matches = Vec::new();
    let max = limit.unwrap_or(usize::MAX);

    let mut stream = storage
        .matches
        .query()
        .filter(format!("PartitionKey eq '{MATCH_PARTITION_KEY}'"))
        .into_stream::<MatchEntity>();

    while let Some(page_result) = stream.next().await {
        let page = page_result.map_err(MatchStorageError::from)?;
        for entity in page.entities {
            if matches.len() >= max {
                break;
            }
            match MatchRecord::try_from(entity) {
                Ok(record) => matches.push(record),
                Err(e) => {
                    // Log and skip malformed entities rather than failing the whole list.
                    tracing::warn!("Skipping match with invalid played_at: {e}");
                }
            }
        }
        if matches.len() >= max {
            break;
        }
    }

    Ok(matches)
}

/// Create (record) a new match.
pub async fn create_match(
    storage: &StorageClient,
    record: MatchRecord,
) -> Result<MatchRecord, MatchStorageError> {
    let entity = MatchEntity::from(record.clone());

    let _: azure_data_tables::operations::InsertEntityResponse<MatchEntity> =
        storage
            .matches
            .insert(&entity)
            .map_err(|e| MatchStorageError::Azure(format!("{e}")))?
            .await
            .map_err(|e| MatchStorageError::Azure(format!("{e}")))?;

    Ok(record)
}

/// Delete a match by its ID (RowKey).
///
/// Used when someone records wrong scores and needs to fix it.
pub async fn delete_match(
    storage: &StorageClient,
    match_id: &str,
) -> Result<(), MatchStorageError> {
    storage
        .matches
        .partition_key_client(MATCH_PARTITION_KEY)
        .entity_client(match_id)
        .delete()
        .await
        .map_err(|e| {
            let msg = format!("{e}");
            if msg.contains("ResourceNotFound") || msg.contains("404") {
                MatchStorageError::NotFound(match_id.to_string())
            } else {
                MatchStorageError::Azure(msg)
            }
        })?;

    Ok(())
}
