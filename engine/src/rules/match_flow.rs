use std::collections::HashMap;
use crate::domain::types::TeamId;

pub const DEFAULT_MATCH_DURATION: f64 = 120.0; // 2 minutes of simulation time

#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    Win(TeamId),
    Draw,
}

pub fn check_match_end(match_timer: f64) -> bool {
    match_timer <= 0.0
}

pub fn determine_winner(scores: &HashMap<TeamId, u32>) -> MatchResult {
    let score_0 = scores.get(&0).copied().unwrap_or(0);
    let score_1 = scores.get(&1).copied().unwrap_or(0);

    if score_0 > score_1 {
        MatchResult::Win(0)
    } else if score_1 > score_0 {
        MatchResult::Win(1)
    } else {
        MatchResult::Draw
    }
}
