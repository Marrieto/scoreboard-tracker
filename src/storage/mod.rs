// storage/mod.rs — Azure Table Storage module.
//
// Provides a client wrapper and CRUD operations for all Azure Table Storage
// tables: players, matches, users, and leagues.

pub mod client;
pub mod leagues;
pub mod matches;
pub mod players;
pub mod users;
