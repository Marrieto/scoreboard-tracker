// models/player.rs — Player struct and Azure Table Storage entity mapping.
//
// Azure Table Storage stores entities as rows with a PartitionKey + RowKey
// composite primary key, plus arbitrary properties. We use serde to map our
// Rust struct to/from the JSON representation that the Azure SDK uses.
//
// For our small team (<10 players), we put all players in a single partition
// ("player") so we can list them all with a single partition query.

use serde::{Deserialize, Serialize};

/// A pickleball player in the system.
///
/// Maps to an Azure Table Storage entity in the "players" table:
/// - PartitionKey: always "player" (single partition for easy listing)
/// - RowKey: a URL-friendly slug like "martin" or "sarah"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// URL-friendly unique identifier (e.g., "martin"). This becomes the RowKey.
    pub id: String,

    /// Display name shown in the UI (e.g., "Martin").
    pub name: String,

    /// Optional fun alias (e.g., "The Dinkmaster"). Can be empty.
    #[serde(default)]
    pub nickname: String,

    /// Emoji used as the player's avatar (e.g., "🏓", "🔥", "💀").
    #[serde(default = "default_avatar")]
    pub avatar_emoji: String,
}

fn default_avatar() -> String {
    "🏓".to_string()
}

/// The shape we store in Azure Table Storage.
///
/// Azure Table Storage requires PartitionKey and RowKey as the first two fields.
/// The SDK serializes/deserializes using serde, so our field names here must
/// match what we want stored in the table.
///
/// We use PascalCase for PartitionKey/RowKey because that's what Azure expects,
/// and snake_case for our custom fields because that's idiomatic Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerEntity {
    /// Always "player" — groups all players in one partition.
    pub partition_key: String,

    /// The player's unique slug (same as Player.id).
    pub row_key: String,

    /// Display name.
    #[serde(rename = "name")]
    pub name: String,

    /// Fun alias / nickname.
    #[serde(rename = "nickname", default)]
    pub nickname: String,

    /// Emoji avatar.
    #[serde(rename = "avatar_emoji", default = "default_avatar")]
    pub avatar_emoji: String,
}

/// The constant partition key we use for all players.
pub const PLAYER_PARTITION_KEY: &str = "player";

impl From<Player> for PlayerEntity {
    /// Convert a domain Player into an Azure Table Storage entity.
    fn from(player: Player) -> Self {
        Self {
            partition_key: PLAYER_PARTITION_KEY.to_string(),
            row_key: player.id,
            name: player.name,
            nickname: player.nickname,
            avatar_emoji: player.avatar_emoji,
        }
    }
}

impl From<PlayerEntity> for Player {
    /// Convert an Azure Table Storage entity back into a domain Player.
    fn from(entity: PlayerEntity) -> Self {
        Self {
            id: entity.row_key,
            name: entity.name,
            nickname: entity.nickname,
            avatar_emoji: entity.avatar_emoji,
        }
    }
}

/// Request body for creating or updating a player.
/// Separate from Player so we control exactly what the API accepts.
#[derive(Debug, Deserialize)]
pub struct CreatePlayerRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default = "default_avatar")]
    pub avatar_emoji: String,
}

/// Request body for updating an existing player (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdatePlayerRequest {
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub avatar_emoji: Option<String>,
}
