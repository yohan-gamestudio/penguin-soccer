use crate::domain::action::Action;
use crate::domain::game_state::TurnPhase;
use crate::domain::types::PlayerId;
use std::collections::HashMap;

pub const PLANNING_DEADLINE: f64 = 30.0;

#[derive(Debug)]
pub enum TurnError {
    NotInPlanningPhase,
    AlreadySubmitted,
    InvalidPlayer,
}

pub fn submit_action(
    phase: &mut TurnPhase,
    actions: &mut HashMap<PlayerId, Action>,
    player_id: PlayerId,
    action: Action,
    player_count: usize,
) -> Result<bool, TurnError> {
    match phase {
        TurnPhase::Planning { submitted, .. } => {
            if submitted.contains(&player_id) {
                return Err(TurnError::AlreadySubmitted);
            }
            submitted.insert(player_id);
            actions.insert(player_id, action);

            Ok(submitted.len() >= player_count)
        }
        _ => Err(TurnError::NotInPlanningPhase),
    }
}

pub fn check_timeout(phase: &mut TurnPhase, dt: f64, player_ids: &[PlayerId], actions: &mut HashMap<PlayerId, Action>) -> bool {
    match phase {
        TurnPhase::Planning { elapsed, deadline, submitted, .. } => {
            *elapsed += dt;
            if *elapsed >= *deadline {
                for &pid in player_ids {
                    if !submitted.contains(&pid) {
                        actions.insert(pid, Action::idle());
                    }
                }
                return true;
            }
            false
        }
        _ => false,
    }
}
