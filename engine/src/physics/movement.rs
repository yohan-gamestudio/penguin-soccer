use crate::domain::ball::Ball;
use crate::domain::penguin::Penguin;

const FRICTION: f64 = 0.98;
const STOP_THRESHOLD: f64 = 1.0;

pub fn apply_movement_penguin(penguin: &mut Penguin, dt: f64) {
    penguin.pos = penguin.pos + penguin.vel * dt;
    penguin.vel = penguin.vel * FRICTION;
    if penguin.vel.length_squared() < STOP_THRESHOLD * STOP_THRESHOLD {
        penguin.vel = crate::domain::types::Vec2::ZERO;
    }
}

pub fn apply_movement_ball(ball: &mut Ball, dt: f64) {
    ball.pos = ball.pos + ball.vel * dt;
    ball.vel = ball.vel * FRICTION;
    if ball.vel.length_squared() < STOP_THRESHOLD * STOP_THRESHOLD {
        ball.vel = crate::domain::types::Vec2::ZERO;
    }
}

pub fn all_stopped(penguins: &[Penguin], ball: &Ball) -> bool {
    let ball_stopped = ball.vel.length_squared() < STOP_THRESHOLD * STOP_THRESHOLD;
    let penguins_stopped = penguins.iter().all(|p| p.vel.length_squared() < STOP_THRESHOLD * STOP_THRESHOLD);
    ball_stopped && penguins_stopped
}
