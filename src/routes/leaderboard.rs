// routes/leaderboard.rs — Leaderboard and stats API handlers.
//
// Stats are computed on-the-fly from match data. With <10 players and a few
// hundred matches at most, this is fast enough without caching.
//
// The leaderboard ranks players by win rate (with a minimum number of games
// to avoid someone being #1 with 1 win and 0 losses).

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::HashMap;

use crate::models::match_record::MatchRecord;
use crate::storage::client::StorageClient;
use crate::storage::matches::{self, MatchStorageError};
use crate::storage::players::{self, PlayerStorageError};

/// A player's entry on the leaderboard.
#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub player_id: String,
    pub player_name: String,
    pub avatar_emoji: String,
    pub nickname: String,
    pub wins: u32,
    pub losses: u32,
    pub total_games: u32,
    pub win_rate: f64,
    /// Current streak: positive = winning, negative = losing.
    pub streak: i32,
}

/// Detailed stats for a single player.
#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub player_id: String,
    pub player_name: String,
    pub avatar_emoji: String,
    pub nickname: String,
    pub wins: u32,
    pub losses: u32,
    pub total_games: u32,
    pub win_rate: f64,
    pub streak: i32,
    /// Best partner: (partner_id, partner_name, wins_together, losses_together)
    pub best_partner: Option<PartnerStats>,
    /// Nemesis: the player they lose to most.
    pub nemesis: Option<RivalryStats>,
    /// Recent matches (last 10).
    pub recent_matches: Vec<MatchRecord>,
}

#[derive(Debug, Serialize)]
pub struct PartnerStats {
    pub partner_id: String,
    pub partner_name: String,
    pub wins: u32,
    pub losses: u32,
}

#[derive(Debug, Serialize)]
pub struct RivalryStats {
    pub opponent_id: String,
    pub opponent_name: String,
    pub wins_against: u32,
    pub losses_against: u32,
}

/// Head-to-head record between two players or pairs.
#[derive(Debug, Serialize)]
pub struct RivalryEntry {
    pub player1_id: String,
    pub player1_name: String,
    pub player2_id: String,
    pub player2_name: String,
    /// Games where player1 was on the winning team and player2 was on the losing team.
    pub player1_wins: u32,
    /// Games where player2 was on the winning team and player1 was on the losing team.
    pub player2_wins: u32,
}

/// Unified error type for leaderboard endpoints.
#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error("{0}")]
    Player(#[from] PlayerStorageError),
    #[error("{0}")]
    Match(#[from] MatchStorageError),
}

impl IntoResponse for StatsError {
    fn into_response(self) -> axum::response::Response {
        match self {
            StatsError::Player(e) => e.into_response(),
            StatsError::Match(e) => e.into_response(),
        }
    }
}

/// GET /api/leaderboard — Ranked player list with stats.
pub async fn get_leaderboard(
    State(storage): State<StorageClient>,
) -> Result<Json<Vec<LeaderboardEntry>>, StatsError> {
    let all_players = players::list_players(&storage).await?;
    let all_matches = matches::list_matches(&storage, None).await?;

    // Count wins/losses per player and track streaks.
    let mut wins: HashMap<&str, u32> = HashMap::new();
    let mut losses: HashMap<&str, u32> = HashMap::new();

    // For streak calculation, we need matches in chronological order per player.
    // all_matches is already sorted newest-first (reverse timestamp RowKey).
    let mut last_results: HashMap<&str, Vec<bool>> = HashMap::new(); // true=win, false=loss

    for m in &all_matches {
        for winner_id in [&m.winner1_id, &m.winner2_id] {
            *wins.entry(winner_id.as_str()).or_default() += 1;
            last_results
                .entry(winner_id.as_str())
                .or_default()
                .push(true);
        }
        for loser_id in [&m.loser1_id, &m.loser2_id] {
            *losses.entry(loser_id.as_str()).or_default() += 1;
            last_results
                .entry(loser_id.as_str())
                .or_default()
                .push(false);
        }
    }

    let mut entries: Vec<LeaderboardEntry> = all_players
        .iter()
        .map(|p| {
            let w = wins.get(p.id.as_str()).copied().unwrap_or(0);
            let l = losses.get(p.id.as_str()).copied().unwrap_or(0);
            let total = w + l;
            let win_rate = if total > 0 {
                w as f64 / total as f64
            } else {
                0.0
            };

            // Calculate current streak from most recent matches.
            // last_results are in newest-first order (from all_matches order).
            let streak = calculate_streak(
                last_results.get(p.id.as_str()).map(|v| v.as_slice()).unwrap_or(&[]),
            );

            LeaderboardEntry {
                player_id: p.id.clone(),
                player_name: p.name.clone(),
                avatar_emoji: p.avatar_emoji.clone(),
                nickname: p.nickname.clone(),
                wins: w,
                losses: l,
                total_games: total,
                win_rate,
                streak,
            }
        })
        .collect();

    // Sort by win rate descending, then by total games descending as tiebreaker.
    entries.sort_by(|a, b| {
        b.win_rate
            .partial_cmp(&a.win_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.total_games.cmp(&a.total_games))
    });

    Ok(Json(entries))
}

/// GET /api/players/:id/stats — Detailed stats for one player.
pub async fn get_player_stats(
    State(storage): State<StorageClient>,
    Path(player_id): Path<String>,
) -> Result<Json<PlayerStats>, StatsError> {
    let player = players::get_player(&storage, &player_id).await?;
    let all_matches = matches::list_matches(&storage, None).await?;

    let mut wins = 0u32;
    let mut losses = 0u32;
    let mut results: Vec<bool> = Vec::new(); // newest-first
    let mut partner_record: HashMap<String, (u32, u32)> = HashMap::new(); // (wins, losses)
    let mut opponent_record: HashMap<String, (u32, u32)> = HashMap::new(); // (wins_against, losses_against)
    let mut recent: Vec<MatchRecord> = Vec::new();

    for m in &all_matches {
        let is_winner = m.winner1_id == player_id || m.winner2_id == player_id;
        let is_loser = m.loser1_id == player_id || m.loser2_id == player_id;

        if !is_winner && !is_loser {
            continue;
        }

        if recent.len() < 10 {
            recent.push(m.clone());
        }

        if is_winner {
            wins += 1;
            results.push(true);

            // Track partner
            let partner = if m.winner1_id == player_id {
                &m.winner2_id
            } else {
                &m.winner1_id
            };
            partner_record
                .entry(partner.clone())
                .or_default()
                .0 += 1;

            // Track opponents
            for opp in [&m.loser1_id, &m.loser2_id] {
                opponent_record
                    .entry(opp.clone())
                    .or_default()
                    .0 += 1;
            }
        } else {
            losses += 1;
            results.push(false);

            // Track partner
            let partner = if m.loser1_id == player_id {
                &m.loser2_id
            } else {
                &m.loser1_id
            };
            partner_record
                .entry(partner.clone())
                .or_default()
                .1 += 1;

            // Track opponents
            for opp in [&m.winner1_id, &m.winner2_id] {
                opponent_record
                    .entry(opp.clone())
                    .or_default()
                    .1 += 1;
            }
        }
    }

    let total = wins + losses;
    let win_rate = if total > 0 {
        wins as f64 / total as f64
    } else {
        0.0
    };
    let streak = calculate_streak(&results);

    // Find best partner (most wins together, minimum 2 games)
    let all_players = players::list_players(&storage).await?;
    let player_names: HashMap<&str, &str> = all_players
        .iter()
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();

    let best_partner = partner_record
        .iter()
        .filter(|(_, (w, l))| w + l >= 2)
        .max_by_key(|(_, (w, _))| *w)
        .map(|(pid, (w, l))| PartnerStats {
            partner_id: pid.clone(),
            partner_name: player_names
                .get(pid.as_str())
                .unwrap_or(&"Unknown")
                .to_string(),
            wins: *w,
            losses: *l,
        });

    // Find nemesis (opponent they lose to most, minimum 2 games)
    let nemesis = opponent_record
        .iter()
        .filter(|(_, (_, l))| *l >= 2)
        .max_by_key(|(_, (_, l))| *l)
        .map(|(oid, (w, l))| RivalryStats {
            opponent_id: oid.clone(),
            opponent_name: player_names
                .get(oid.as_str())
                .unwrap_or(&"Unknown")
                .to_string(),
            wins_against: *w,
            losses_against: *l,
        });

    Ok(Json(PlayerStats {
        player_id: player.id,
        player_name: player.name,
        avatar_emoji: player.avatar_emoji,
        nickname: player.nickname,
        wins,
        losses,
        total_games: total,
        win_rate,
        streak,
        best_partner,
        nemesis,
        recent_matches: recent,
    }))
}

/// GET /api/rivalries — Head-to-head records between all player pairs.
pub async fn get_rivalries(
    State(storage): State<StorageClient>,
) -> Result<Json<Vec<RivalryEntry>>, StatsError> {
    let all_players = players::list_players(&storage).await?;
    let all_matches = matches::list_matches(&storage, None).await?;

    let player_names: HashMap<&str, &str> = all_players
        .iter()
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();

    // Count head-to-head: key is (player_a, player_b) where a < b lexicographically.
    // Value is (a_wins_over_b, b_wins_over_a).
    let mut h2h: HashMap<(String, String), (u32, u32)> = HashMap::new();

    for m in &all_matches {
        // For each winner-loser pair
        for winner in [&m.winner1_id, &m.winner2_id] {
            for loser in [&m.loser1_id, &m.loser2_id] {
                let (a, b, winner_is_a) = if winner < loser {
                    (winner.clone(), loser.clone(), true)
                } else {
                    (loser.clone(), winner.clone(), false)
                };

                let entry = h2h.entry((a, b)).or_default();
                if winner_is_a {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
            }
        }
    }

    let mut rivalries: Vec<RivalryEntry> = h2h
        .into_iter()
        .filter(|(_, (w1, w2))| w1 + w2 >= 2) // Only show pairs with at least 2 games
        .map(|((p1, p2), (p1_wins, p2_wins))| RivalryEntry {
            player1_name: player_names
                .get(p1.as_str())
                .unwrap_or(&"Unknown")
                .to_string(),
            player1_id: p1,
            player2_name: player_names
                .get(p2.as_str())
                .unwrap_or(&"Unknown")
                .to_string(),
            player2_id: p2,
            player1_wins: p1_wins,
            player2_wins: p2_wins,
        })
        .collect();

    // Sort by total games descending for the most active rivalries first.
    rivalries.sort_by(|a, b| {
        (b.player1_wins + b.player2_wins).cmp(&(a.player1_wins + a.player2_wins))
    });

    Ok(Json(rivalries))
}

/// Calculate the current streak from a list of results (newest first).
///
/// Returns positive for a winning streak, negative for a losing streak.
/// E.g., [true, true, false, ...] → 2 (two wins in a row).
///       [false, false, false, true, ...] → -3 (three losses in a row).
fn calculate_streak(results: &[bool]) -> i32 {
    if results.is_empty() {
        return 0;
    }

    let first = results[0];
    let count = results.iter().take_while(|&&r| r == first).count() as i32;

    if first { count } else { -count }
}
