// models/mod.rs — Data model module.
//
// Defines the core domain types (Player, MatchRecord, User, League) and their
// mappings to/from Azure Table Storage entities. Each model has its own file
// with detailed comments explaining the domain concept and serialization strategy.

pub mod league;
pub mod match_record;
pub mod player;
pub mod user;
