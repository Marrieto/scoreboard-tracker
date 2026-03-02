// storage/users.rs — User CRUD operations against Azure Table Storage.
//
// This module handles persistence for User records. Users are stored in the
// "users" table with PartitionKey "user" and RowKey equal to the Microsoft OID.
//
// Key operations:
//   - list_users:      Get all users (admin functionality).
//   - get_user:        Get a single user by OID.
//   - upsert_user:     Create or update a user (used on login callback).
//   - update_user_role: Change a user's role (admin only).
//   - link_player:     Link/unlink a player profile to a user.
//   - count_users:     Count total users (used for first-user-is-admin logic).
//
// Error handling uses the `thiserror` crate to define a custom error enum.
// Each variant maps to a different HTTP status code in the route handlers.

use futures::StreamExt;

use crate::models::user::{User, UserEntity, USER_PARTITION_KEY};
use crate::storage::client::StorageClient;

/// Errors that can occur during user storage operations.
///
/// The `#[derive(thiserror::Error)]` macro generates `Display` and `Error` trait
/// implementations automatically. The `#[error("...")]` attribute defines the
/// display format for each variant.
#[derive(Debug, thiserror::Error)]
pub enum UserStorageError {
    /// The requested user was not found in storage.
    #[error("User '{0}' not found")]
    NotFound(String),

    /// The caller doesn't have permission for this operation.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// An unexpected error from the Azure SDK.
    #[error("Azure Table Storage error: {0}")]
    Azure(String),
}

/// Convert Azure SDK errors into our domain error type.
///
/// This `From` implementation lets us use the `?` operator on Azure SDK calls,
/// which automatically converts `azure_core::Error` into `UserStorageError`.
impl From<azure_core::Error> for UserStorageError {
    fn from(e: azure_core::Error) -> Self {
        let msg = format!("{e}");
        if msg.contains("ResourceNotFound") || msg.contains("404") {
            UserStorageError::NotFound("(unknown)".to_string())
        } else {
            UserStorageError::Azure(msg)
        }
    }
}

/// List all users in the system.
///
/// Returns all users from the "user" partition. Since we expect <50 users
/// for a small team, we don't need pagination — just collect everything.
///
/// The Azure SDK returns a paginated `Stream` (an async iterator), so we
/// use `futures::StreamExt::next()` to iterate through pages.
pub async fn list_users(storage: &StorageClient) -> Result<Vec<User>, UserStorageError> {
    let mut users = Vec::new();

    // Query all entities in the "user" partition.
    let mut stream = storage
        .users
        .query()
        .filter(format!("PartitionKey eq '{USER_PARTITION_KEY}'"))
        .into_stream::<UserEntity>();

    // Iterate through pages of results (usually just one page for small datasets).
    while let Some(page_result) = stream.next().await {
        let page = page_result.map_err(UserStorageError::from)?;
        for entity in page.entities {
            // TryFrom can fail if the created_at date is malformed.
            // We log and skip malformed entities rather than failing the whole list.
            match User::try_from(entity) {
                Ok(user) => users.push(user),
                Err(e) => {
                    tracing::warn!("Skipping user with invalid created_at: {e}");
                }
            }
        }
    }

    Ok(users)
}

/// Get a single user by their OID (RowKey).
///
/// Uses the point-read API (partition key + row key) which is the fastest
/// possible query in Azure Table Storage — O(1) lookup.
pub async fn get_user(
    storage: &StorageClient,
    oid: &str,
) -> Result<User, UserStorageError> {
    let response = storage
        .users
        .partition_key_client(USER_PARTITION_KEY)
        .entity_client(oid)
        .get::<UserEntity>()
        .await
        .map_err(|e| {
            let msg = format!("{e}");
            if msg.contains("ResourceNotFound") || msg.contains("404") {
                UserStorageError::NotFound(oid.to_string())
            } else {
                UserStorageError::Azure(msg)
            }
        })?;

    User::try_from(response.entity)
        .map_err(|e| UserStorageError::Azure(format!("Failed to parse user: {e}")))
}

/// Create or update a user (upsert).
///
/// This is called on every login callback. If the user already exists, their
/// name and email are updated (in case they changed in Azure AD). If they're
/// new, a fresh record is created.
///
/// We use `insert_or_replace` which is Azure Table Storage's upsert operation.
/// It doesn't require an ETag, so there's no risk of a conflict error.
pub async fn upsert_user(
    storage: &StorageClient,
    user: User,
) -> Result<User, UserStorageError> {
    let entity = UserEntity::from(user.clone());

    storage
        .users
        .partition_key_client(USER_PARTITION_KEY)
        .entity_client(&entity.row_key)
        .insert_or_replace(&entity)
        .map_err(|e| UserStorageError::Azure(format!("{e}")))?
        .await
        .map_err(|e| UserStorageError::Azure(format!("{e}")))?;

    Ok(user)
}

/// Update a user's role (admin or user).
///
/// This performs a read-modify-write cycle: fetch the current user, change
/// their role, and write back. This preserves all other fields.
pub async fn update_user_role(
    storage: &StorageClient,
    oid: &str,
    new_role: &str,
) -> Result<User, UserStorageError> {
    // Validate the role value before doing any storage operations.
    if new_role != "admin" && new_role != "user" {
        return Err(UserStorageError::Azure(format!(
            "Invalid role '{new_role}': must be 'admin' or 'user'"
        )));
    }

    // Fetch the current user to preserve their other fields.
    let mut user = get_user(storage, oid).await?;
    user.role = new_role.to_string();

    // Write back the updated user.
    upsert_user(storage, user).await
}

/// Link or unlink a player profile to/from a user.
///
/// When `player_id` is `Some(id)`, the user is linked to that player.
/// When `player_id` is `None`, the link is removed.
pub async fn link_player(
    storage: &StorageClient,
    oid: &str,
    player_id: Option<String>,
) -> Result<User, UserStorageError> {
    let mut user = get_user(storage, oid).await?;
    user.player_id = player_id;

    upsert_user(storage, user).await
}

/// Count the total number of users in the system.
///
/// This is used by the auth callback to implement "first user is admin" logic.
/// Rather than listing all users and counting (which returns full entities),
/// we just iterate through the stream and count entries. With <50 users this
/// is efficient enough.
pub async fn count_users(storage: &StorageClient) -> Result<usize, UserStorageError> {
    let mut count = 0;

    let mut stream = storage
        .users
        .query()
        .filter(format!("PartitionKey eq '{USER_PARTITION_KEY}'"))
        .into_stream::<UserEntity>();

    while let Some(page_result) = stream.next().await {
        let page = page_result.map_err(UserStorageError::from)?;
        count += page.entities.len();
    }

    Ok(count)
}
