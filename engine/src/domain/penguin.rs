use crate::domain::types::{PlayerId, TeamId, Vec2};

pub const PENGUIN_RADIUS: f64 = 15.0;
pub const PENGUIN_MASS: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Penguin {
    pub id: PlayerId,
    pub team: TeamId,
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f64,
    pub mass: f64,
}

impl Penguin {
    pub fn new(id: PlayerId, team: TeamId, pos: Vec2) -> Self {
        Self {
            id,
            team,
            pos,
            vel: Vec2::ZERO,
            radius: PENGUIN_RADIUS,
            mass: PENGUIN_MASS,
        }
    }
}
