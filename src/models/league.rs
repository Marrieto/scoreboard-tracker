// models/league.rs — League struct and Azure Table Storage entity mapping.
//
// A League represents a time-bounded season that groups matches together.
// Leagues let players track stats for specific time periods (e.g., "Spring 2026"
// or "Office Tournament Q1") while still being able to view all-time stats.
//
// Key design decisions:
//   - Leagues have a simple Active/Closed lifecycle (no draft, no scheduled).
//   - Only one league needs to be active at a time per practical usage, but
//     the system doesn't enforce this — it's a UI convention.
//   - Matches reference leagues by ID (optional field), so existing matches
//     without a league_id still work (they show up in "All Time" stats).
//   - League IDs are URL-friendly slugs (like "spring-2026"), used as RowKeys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A league (season) that groups matches together.
///
/// Leagues are created by any authenticated user and can be closed by the
/// creator or an admin. Once closed, no new matches should be assigned to it
/// (enforced by UI convention, not backend validation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct League {
    /// URL-friendly unique identifier (e.g., "spring-2026"). This is the RowKey.
    pub id: String,

    /// Human-readable name (e.g., "Spring 2026 Season").
    pub name: String,

    /// Optional description (e.g., "Casual league, March through May").
    #[serde(default)]
    pub description: String,

    /// The OID of the user who created this league. Used for authorization —
    /// only the creator (or an admin) can edit or close the league.
    pub created_by: String,

    /// League status: "active" or "closed".
    /// Stored as a String for the same reason as User.role — Azure Table Storage
    /// works best with flat string properties.
    pub status: String,

    /// When the league was created (ISO 8601).
    pub created_at: DateTime<Utc>,

    /// When the league was closed, if applicable.
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
}

/// Azure Table Storage entity for a league.
///
/// Same pattern as PlayerEntity and MatchEntity — PascalCase for system fields,
/// explicit renames for custom fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LeagueEntity {
    /// Always "league" — groups all leagues in one partition.
    pub partition_key: String,

    /// The league's unique slug ID.
    pub row_key: String,

    /// Human-readable name.
    #[serde(rename = "name")]
    pub name: String,

    /// Optional description.
    #[serde(rename = "description", default)]
    pub description: String,

    /// OID of the creator.
    #[serde(rename = "created_by")]
    pub created_by: String,

    /// Status string: "active" or "closed".
    #[serde(rename = "status")]
    pub status: String,

    /// When the league was created (ISO 8601 string).
    #[serde(rename = "created_at")]
    pub created_at: String,

    /// When the league was closed (ISO 8601 string), if applicable.
    /// `#[serde(default)]` handles the case where this field doesn't exist
    /// in storage (i.e., the league is still active).
    #[serde(rename = "closed_at", default)]
    pub closed_at: Option<String>,
}

/// The constant partition key for all leagues.
pub const LEAGUE_PARTITION_KEY: &str = "league";

/// Convert a domain League into an Azure Table Storage entity.
impl From<League> for LeagueEntity {
    fn from(league: League) -> Self {
        Self {
            partition_key: LEAGUE_PARTITION_KEY.to_string(),
            row_key: league.id,
            name: league.name,
            description: league.description,
            created_by: league.created_by,
            status: league.status,
            created_at: league.created_at.to_rfc3339(),
            closed_at: league.closed_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Convert an Azure Table Storage entity back into a domain League.
///
/// Uses `TryFrom` because date parsing can fail on malformed stored values.
impl TryFrom<LeagueEntity> for League {
    type Error = chrono::ParseError;

    fn try_from(entity: LeagueEntity) -> Result<Self, Self::Error> {
        let created_at = DateTime::parse_from_rfc3339(&entity.created_at)?
            .with_timezone(&Utc);
        let closed_at = entity
            .closed_at
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()?;
        Ok(Self {
            id: entity.row_key,
            name: entity.name,
            description: entity.description,
            created_by: entity.created_by,
            status: entity.status,
            created_at,
            closed_at,
        })
    }
}

/// Request body for creating a new league.
///
/// The `created_by` field is NOT included here — it's set from the authenticated
/// user's session claims in the handler. This prevents spoofing the creator.
#[derive(Debug, Deserialize)]
pub struct CreateLeagueRequest {
    /// URL-friendly slug ID (e.g., "spring-2026").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
}

/// Request body for updating an existing league.
///
/// Only name and description can be updated — status is changed via the
/// dedicated close endpoint, and the creator can't be changed.
#[derive(Debug, Deserialize)]
pub struct UpdateLeagueRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}
