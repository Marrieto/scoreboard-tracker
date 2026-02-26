// models/match_record.rs — Match struct and Azure Table Storage entity mapping.
//
// Each match records a doubles (2v2) pickleball game result. We store them in
// Azure Table Storage with a reverse-timestamp RowKey so that querying the
// partition returns matches in newest-first order by default.
//
// Reverse timestamp trick:
//   Azure Table Storage sorts RowKeys in ascending lexicographic order.
//   By using (MAX_TIMESTAMP - actual_timestamp) as a prefix, newer matches
//   get smaller RowKeys and appear first in query results. This avoids
//   needing to sort client-side.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum timestamp value for the reverse-timestamp trick.
/// We use the year 9999 in milliseconds as our ceiling.
const MAX_TIMESTAMP_MS: i64 = 253_402_300_799_999;

/// A recorded pickleball match (doubles: 2v2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    /// Unique match ID (the RowKey from Azure, which includes the reverse timestamp).
    pub id: String,

    /// Player IDs of the winning team.
    pub winner1_id: String,
    pub winner2_id: String,

    /// Player IDs of the losing team.
    pub loser1_id: String,
    pub loser2_id: String,

    /// Optional scores.
    pub winner_score: Option<i32>,
    pub loser_score: Option<i32>,

    /// Optional trash talk or match notes.
    #[serde(default)]
    pub comment: String,

    /// Who submitted this match result (from auth).
    pub recorded_by: String,

    /// When the match was played (ISO 8601).
    pub played_at: DateTime<Utc>,
}

/// Azure Table Storage entity for a match.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MatchEntity {
    pub partition_key: String,
    pub row_key: String,

    #[serde(rename = "winner1_id")]
    pub winner1_id: String,
    #[serde(rename = "winner2_id")]
    pub winner2_id: String,
    #[serde(rename = "loser1_id")]
    pub loser1_id: String,
    #[serde(rename = "loser2_id")]
    pub loser2_id: String,
    #[serde(rename = "winner_score")]
    pub winner_score: Option<i32>,
    #[serde(rename = "loser_score")]
    pub loser_score: Option<i32>,
    #[serde(rename = "comment", default)]
    pub comment: String,
    #[serde(rename = "recorded_by")]
    pub recorded_by: String,
    #[serde(rename = "played_at")]
    pub played_at: String,
}

/// The constant partition key for all matches.
pub const MATCH_PARTITION_KEY: &str = "match";

/// Generate a RowKey that sorts newest-first in Azure Table Storage.
///
/// Format: `{reverse_timestamp}_{uuid}`
///
/// The reverse timestamp ensures that newer matches have lexicographically
/// smaller RowKeys, so Azure's default ascending sort returns newest first.
/// The UUID suffix guarantees uniqueness even for matches at the same millisecond.
pub fn generate_match_row_key(played_at: &DateTime<Utc>) -> String {
    let ms = played_at.timestamp_millis();
    let reverse = MAX_TIMESTAMP_MS - ms;
    let uuid = Uuid::new_v4();
    // Zero-pad the reverse timestamp to 20 digits so lexicographic sort works correctly.
    format!("{reverse:020}_{uuid}")
}

impl MatchRecord {
    /// Create a new MatchRecord, generating the reverse-timestamp ID.
    pub fn new(
        winner1_id: String,
        winner2_id: String,
        loser1_id: String,
        loser2_id: String,
        winner_score: Option<i32>,
        loser_score: Option<i32>,
        comment: String,
        recorded_by: String,
        played_at: DateTime<Utc>,
    ) -> Self {
        let id = generate_match_row_key(&played_at);
        Self {
            id,
            winner1_id,
            winner2_id,
            loser1_id,
            loser2_id,
            winner_score,
            loser_score,
            comment,
            recorded_by,
            played_at,
        }
    }
}

impl From<MatchRecord> for MatchEntity {
    fn from(m: MatchRecord) -> Self {
        Self {
            partition_key: MATCH_PARTITION_KEY.to_string(),
            row_key: m.id,
            winner1_id: m.winner1_id,
            winner2_id: m.winner2_id,
            loser1_id: m.loser1_id,
            loser2_id: m.loser2_id,
            winner_score: m.winner_score,
            loser_score: m.loser_score,
            comment: m.comment,
            recorded_by: m.recorded_by,
            played_at: m.played_at.to_rfc3339(),
        }
    }
}

impl TryFrom<MatchEntity> for MatchRecord {
    type Error = chrono::ParseError;

    fn try_from(entity: MatchEntity) -> Result<Self, Self::Error> {
        let played_at = DateTime::parse_from_rfc3339(&entity.played_at)?
            .with_timezone(&Utc);
        Ok(Self {
            id: entity.row_key,
            winner1_id: entity.winner1_id,
            winner2_id: entity.winner2_id,
            loser1_id: entity.loser1_id,
            loser2_id: entity.loser2_id,
            winner_score: entity.winner_score,
            loser_score: entity.loser_score,
            comment: entity.comment,
            recorded_by: entity.recorded_by,
            played_at,
        })
    }
}

/// Request body for recording a new match.
#[derive(Debug, Deserialize)]
pub struct CreateMatchRequest {
    pub winner1_id: String,
    pub winner2_id: String,
    pub loser1_id: String,
    pub loser2_id: String,
    pub winner_score: Option<i32>,
    pub loser_score: Option<i32>,
    #[serde(default)]
    pub comment: String,
    /// Optional: when the match was played. Defaults to now if omitted.
    pub played_at: Option<DateTime<Utc>>,
}
