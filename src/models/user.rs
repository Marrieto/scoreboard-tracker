// models/user.rs — User struct and Azure Table Storage entity mapping.
//
// Users represent authenticated people in the system. Each user is identified
// by their Microsoft OID (Object ID) — a GUID assigned by Azure AD / Entra ID
// that uniquely identifies a person across the organization.
//
// User roles:
//   - "admin": Can manage other users, edit any match, manage any league.
//   - "user":  Can record matches, create leagues, edit matches they played in.
//
// The first user to log in automatically becomes an admin (bootstrap logic in
// the auth callback). Subsequent users get the "user" role by default.
//
// Users can optionally be linked to a Player profile via `player_id`. This
// allows the system to know which matches a user participated in (for edit
// permissions) and to show a "claim profile" prompt in the UI.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user in the system, created automatically on first login.
///
/// This is separate from `Player` — a Player is a game participant (could be
/// created by anyone), while a User is an authenticated person. Linking them
/// via `player_id` connects the two concepts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The Microsoft OID (Object ID) — a GUID that uniquely identifies this
    /// user in the Azure AD tenant. This is the RowKey in Azure Table Storage.
    pub oid: String,

    /// Display name from Microsoft (e.g., "Martin Smith").
    pub name: String,

    /// Email address from Microsoft (e.g., "martin@company.com").
    pub email: String,

    /// User role: "admin" or "user".
    /// Stored as a String rather than an enum for Azure Table Storage compatibility
    /// (Azure stores string properties natively, and serde's default enum
    /// serialization produces JSON like `{"Admin": null}` which is awkward for
    /// table storage). We validate the values in application logic instead.
    pub role: String,

    /// Optional link to a Player profile. When set, this user is considered
    /// a participant in any match involving this player_id, which grants them
    /// edit permissions for those matches.
    #[serde(default)]
    pub player_id: Option<String>,

    /// When this user first logged in (ISO 8601).
    pub created_at: DateTime<Utc>,
}

/// Azure Table Storage entity for a user.
///
/// Azure Table Storage requires PascalCase for PartitionKey and RowKey.
/// We use `#[serde(rename_all = "PascalCase")]` for those two system fields,
/// then explicit `#[serde(rename = "...")]` for our custom fields to keep them
/// snake_case in storage (matching our convention from PlayerEntity/MatchEntity).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserEntity {
    /// Always "user" — groups all users in one partition for easy listing.
    pub partition_key: String,

    /// The user's Microsoft OID (same as User.oid).
    pub row_key: String,

    /// Display name.
    #[serde(rename = "name")]
    pub name: String,

    /// Email address.
    #[serde(rename = "email")]
    pub email: String,

    /// Role string: "admin" or "user".
    #[serde(rename = "role")]
    pub role: String,

    /// Optional link to a Player profile ID.
    /// `#[serde(default)]` ensures backward compatibility — if this field is
    /// missing from an existing entity in storage, it deserializes as `None`.
    #[serde(rename = "player_id", default)]
    pub player_id: Option<String>,

    /// When this user was created (ISO 8601 string in storage).
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// The constant partition key for all users — keeps them in a single partition
/// so we can list all users with one query (fine for small teams).
pub const USER_PARTITION_KEY: &str = "user";

/// Convert a domain User into an Azure Table Storage entity.
///
/// The `From` trait in Rust enables zero-cost conversions between types.
/// Implementing `From<User> for UserEntity` also automatically gives us
/// `Into<UserEntity>` for `User` (the compiler provides the reverse direction).
impl From<User> for UserEntity {
    fn from(user: User) -> Self {
        Self {
            partition_key: USER_PARTITION_KEY.to_string(),
            row_key: user.oid,
            name: user.name,
            email: user.email,
            role: user.role,
            player_id: user.player_id,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

/// Convert an Azure Table Storage entity back into a domain User.
///
/// This uses `TryFrom` instead of `From` because parsing the `created_at`
/// date string can fail (if the stored value is malformed). Using `TryFrom`
/// forces callers to handle the error case explicitly with `?` or `.unwrap()`.
impl TryFrom<UserEntity> for User {
    type Error = chrono::ParseError;

    fn try_from(entity: UserEntity) -> Result<Self, Self::Error> {
        let created_at = DateTime::parse_from_rfc3339(&entity.created_at)?
            .with_timezone(&Utc);
        Ok(Self {
            oid: entity.row_key,
            name: entity.name,
            email: entity.email,
            role: entity.role,
            player_id: entity.player_id,
            created_at,
        })
    }
}

/// Request body for updating a user's role (admin-only operation).
///
/// We use a separate request type rather than accepting the full User struct
/// to limit what the API caller can change. This is the "parse, don't validate"
/// pattern — the type system ensures only the role can be modified via this endpoint.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    /// The new role: must be "admin" or "user".
    pub role: String,
}

/// Request body for linking or unlinking a player profile to a user.
///
/// Setting `player_id` to `None` (or omitting it) unlinks the player.
/// Setting it to `Some("player-slug")` links the user to that player.
#[derive(Debug, Deserialize)]
pub struct LinkPlayerRequest {
    /// The player ID to link, or null/absent to unlink.
    pub player_id: Option<String>,
}
