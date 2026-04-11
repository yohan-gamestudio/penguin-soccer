use crate::domain::ball::Ball;
use crate::domain::field::Field;
use crate::domain::penguin::Penguin;
use crate::domain::types::Vec2;

pub fn collide_circle_wall(pos: &mut Vec2, vel: &mut Vec2, radius: f64, field: &Field) {
    // Left wall
    if pos.x - radius < 0.0 {
        pos.x = radius;
        vel.x = vel.x.abs();
    }
    // Right wall
    if pos.x + radius > field.width {
        pos.x = field.width - radius;
        vel.x = -vel.x.abs();
    }
    // Top wall
    if pos.y - radius < 0.0 {
        pos.y = radius;
        vel.y = vel.y.abs();
    }
    // Bottom wall
    if pos.y + radius > field.height {
        pos.y = field.height - radius;
        vel.y = -vel.y.abs();
    }
}

pub fn collide_penguin_wall(penguin: &mut Penguin, field: &Field) {
    collide_circle_wall(&mut penguin.pos, &mut penguin.vel, penguin.radius, field);
}

pub fn collide_ball_wall(ball: &mut Ball, field: &Field) {
    // Top wall
    if ball.pos.y - ball.radius < 0.0 {
        ball.pos.y = ball.radius;
        ball.vel.y = ball.vel.y.abs();
    }
    // Bottom wall
    if ball.pos.y + ball.radius > field.height {
        ball.pos.y = field.height - ball.radius;
        ball.vel.y = -ball.vel.y.abs();
    }

    // Left/right walls — skip bouncing when ball is within a goal opening
    let in_left_goal = field.goals.iter().any(|g| {
        g.team == 0
            && ball.pos.y >= g.center.y - g.width / 2.0
            && ball.pos.y <= g.center.y + g.width / 2.0
    });
    let in_right_goal = field.goals.iter().any(|g| {
        g.team == 1
            && ball.pos.y >= g.center.y - g.width / 2.0
            && ball.pos.y <= g.center.y + g.width / 2.0
    });

    // Left wall (only bounce if NOT in left goal area)
    if !in_left_goal && ball.pos.x - ball.radius < 0.0 {
        ball.pos.x = ball.radius;
        ball.vel.x = ball.vel.x.abs();
    }
    // Right wall (only bounce if NOT in right goal area)
    if !in_right_goal && ball.pos.x + ball.radius > field.width {
        ball.pos.x = field.width - ball.radius;
        ball.vel.x = -ball.vel.x.abs();
    }
}

pub fn collide_circles(
    pos_a: &mut Vec2, vel_a: &mut Vec2, mass_a: f64, radius_a: f64,
    pos_b: &mut Vec2, vel_b: &mut Vec2, mass_b: f64, radius_b: f64,
) {
    let delta = *pos_b - *pos_a;
    let dist_sq = delta.length_squared();
    let min_dist = radius_a + radius_b;

    if dist_sq >= min_dist * min_dist || dist_sq < 1e-10 {
        return;
    }

    let dist = dist_sq.sqrt();
    let normal = Vec2::new(delta.x / dist, delta.y / dist);

    // Separate overlapping circles
    let overlap = min_dist - dist;
    let total_mass = mass_a + mass_b;
    *pos_a = *pos_a - normal * (overlap * mass_b / total_mass);
    *pos_b = *pos_b + normal * (overlap * mass_a / total_mass);

    // Elastic collision response
    let relative_vel = *vel_a - *vel_b;
    let vel_along_normal = relative_vel.dot(normal);

    if vel_along_normal <= 0.0 {
        return; // Already separating
    }

    let restitution = 0.9;
    let impulse = (1.0 + restitution) * vel_along_normal / total_mass;

    *vel_a = *vel_a - normal * (impulse * mass_b);
    *vel_b = *vel_b + normal * (impulse * mass_a);
}

pub fn collide_penguin_penguin(a: &mut Penguin, b: &mut Penguin) {
    collide_circles(
        &mut a.pos, &mut a.vel, a.mass, a.radius,
        &mut b.pos, &mut b.vel, b.mass, b.radius,
    );
}

pub fn collide_penguin_ball(penguin: &mut Penguin, ball: &mut Ball) {
    collide_circles(
        &mut penguin.pos, &mut penguin.vel, penguin.mass, penguin.radius,
        &mut ball.pos, &mut ball.vel, ball.mass, ball.radius,
    );
}
