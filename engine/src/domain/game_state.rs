use std::collections::HashMap;

use crate::domain::ball::Ball;
use crate::domain::field::Field;
use crate::domain::penguin::Penguin;
use crate::domain::types::{PlayerId, TeamId};

#[derive(Debug, Clone)]
pub struct GameState {
    pub penguins: Vec<Penguin>,
    pub ball: Ball,
    pub field: Field,
    pub scores: HashMap<TeamId, u32>,
    pub match_timer: f64,
    pub phase: TurnPhase,
}

#[derive(Debug, Clone)]
pub enum TurnPhase {
    Planning {
        submitted: std::collections::HashSet<PlayerId>,
        elapsed: f64,
        deadline: f64,
    },
    Simulating,
    Resolving,
}

impl TurnPhase {
    pub fn new_planning(deadline: f64) -> Self {
        TurnPhase::Planning {
            submitted: std::collections::HashSet::new(),
            elapsed: 0.0,
            deadline,
        }
    }
}
