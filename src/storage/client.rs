// storage/client.rs — Azure Table Storage client wrapper.
//
// Azure Table Storage is a NoSQL key-value store that's cheap, fast, and
// requires zero infrastructure management. It's perfect for our small dataset
// (a handful of players and a few hundred matches at most).
//
// The `azure_data_tables` crate provides a typed client that serializes/
// deserializes entities via serde. We wrap it in our own `StorageClient` to
// keep Azure-specific details out of the rest of the codebase.
//
// Connection: We authenticate using a Storage Account name + access key
// (the simplest approach). In production you might use Managed Identity
// or Azure AD tokens instead.

use azure_data_tables::prelude::*;
use azure_storage::StorageCredentials;

use crate::config::AppConfig;

/// Names of the Azure Table Storage tables we use.
const PLAYERS_TABLE: &str = "players";
const MATCHES_TABLE: &str = "matches";

/// Wrapper around Azure Table Storage that provides access to our two tables.
///
/// This is cheap to clone (the inner client is Arc-based), so we pass it
/// around freely in Axum's State extractor.
#[derive(Clone)]
pub struct StorageClient {
    /// Client for the "players" table.
    pub players: TableClient,
    /// Client for the "matches" table.
    pub matches: TableClient,
}

impl StorageClient {
    /// Create a new StorageClient from the app configuration.
    ///
    /// This doesn't make any network calls yet — table clients are lazy.
    /// The first actual operation (query, insert, etc.) will establish the
    /// connection.
    pub fn new(config: &AppConfig) -> Self {
        // Create credentials from the storage account access key.
        // This is the simplest auth method for Azure Table Storage.
        let credentials = StorageCredentials::access_key(
            config.azure_storage_account.clone(),
            config.azure_storage_access_key.clone(),
        );

        // Create the top-level service client for this storage account.
        let service_client = TableServiceClient::new(
            config.azure_storage_account.clone(),
            credentials,
        );

        // Get typed table clients for each of our tables.
        let players = service_client.table_client(PLAYERS_TABLE);
        let matches = service_client.table_client(MATCHES_TABLE);

        Self { players, matches }
    }

    /// Ensure our tables exist in Azure Table Storage.
    ///
    /// Azure Table Storage requires tables to be created before use. This method
    /// creates them if they don't exist (idempotent). Call this once at startup.
    ///
    /// The "409 Conflict" response from Azure when a table already exists is
    /// handled gracefully — we just ignore it.
    pub async fn ensure_tables_exist(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Try to create each table. If it already exists, Azure returns 409
        // which the SDK surfaces as an error — we check for that and ignore it.
        for (name, client) in [
            (PLAYERS_TABLE, &self.players),
            (MATCHES_TABLE, &self.matches),
        ] {
            match client.create().await {
                Ok(_) => tracing::info!("Created table '{name}'"),
                Err(e) => {
                    // Azure returns HTTP 409 (Conflict) if the table already exists.
                    // That's fine — we just want to ensure it's there.
                    let err_string = format!("{e}");
                    if err_string.contains("TableAlreadyExists")
                        || err_string.contains("409")
                    {
                        tracing::debug!("Table '{name}' already exists");
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Ok(())
    }
}
