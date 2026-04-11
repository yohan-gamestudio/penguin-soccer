use crate::domain::types::Vec2;

pub const BALL_RADIUS: f64 = 10.0;
pub const BALL_MASS: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f64,
    pub mass: f64,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            radius: BALL_RADIUS,
            mass: BALL_MASS,
        }
    }
}
