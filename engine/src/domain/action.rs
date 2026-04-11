use crate::domain::types::Vec2;

pub const MAX_POWER: f64 = 500.0;

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub direction: Vec2,
    pub power: f64,
}

impl Action {
    pub fn new(direction: Vec2, power: f64) -> Self {
        let power = power.clamp(0.0, 1.0);
        let direction = direction.normalized();
        Self { direction, power }
    }

    pub fn idle() -> Self {
        Self {
            direction: Vec2::ZERO,
            power: 0.0,
        }
    }

    pub fn to_velocity(&self) -> Vec2 {
        self.direction * (self.power * MAX_POWER)
    }
}
