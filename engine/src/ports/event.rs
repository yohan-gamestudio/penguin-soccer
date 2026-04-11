use crate::domain::game_state::TurnPhase;
use crate::domain::types::TeamId;
use crate::rules::match_flow::MatchResult;

#[derive(Debug, Clone)]
pub enum GameEvent {
    PhaseChanged(TurnPhase),
    GoalScored { scoring_team: TeamId },
    MatchEnded(MatchResult),
}

pub trait GameEventPort {
    fn on_event(&mut self, event: &GameEvent);
}
