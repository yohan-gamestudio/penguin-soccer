use crate::domain::ball::Ball;
use crate::domain::field::Field;
use crate::domain::penguin::Penguin;
use crate::domain::types::Vec2;

const CORNER_RADIUS: f64 = 30.0; // 3 world units * 10 scale = 30 engine units

fn collide_circle_corner(pos: &mut Vec2, vel: &mut Vec2, radius: f64, field: &Field) {
    let corners = [
        Vec2::new(CORNER_RADIUS, CORNER_RADIUS),                              // top-left
        Vec2::new(field.width - CORNER_RADIUS, CORNER_RADIUS),                // top-right
        Vec2::new(field.width - CORNER_RADIUS, field.height - CORNER_RADIUS), // bottom-right
        Vec2::new(CORNER_RADIUS, field.height - CORNER_RADIUS),               // bottom-left
    ];

    for corner in &corners {
        // Only check if we're in the corner region
        let in_corner_x = pos.x < CORNER_RADIUS || pos.x > field.width - CORNER_RADIUS;
        let in_corner_y = pos.y < CORNER_RADIUS || pos.y > field.height - CORNER_RADIUS;
        if !in_corner_x || !in_corner_y {
            continue;
        }

        let delta = Vec2::new(pos.x - corner.x, pos.y - corner.y);
        let dist = delta.length();
        if dist < 1e-10 {
            continue;
        }

        // Check if circle is outside the rounded corner
        if dist + radius > CORNER_RADIUS {
            let normal = Vec2::new(delta.x / dist, delta.y / dist);
            // Push circle inside
            let penetration = dist + radius - CORNER_RADIUS;
            pos.x += normal.x * penetration;
            pos.y += normal.y * penetration;
            // Reflect velocity
            let dot = vel.x * normal.x + vel.y * normal.y;
            if dot > 0.0 {
                vel.x -= 2.0 * dot * normal.x;
                vel.y -= 2.0 * dot * normal.y;
            }
        }
    }
}

pub fn collide_circle_wall(pos: &mut Vec2, vel: &mut Vec2, radius: f64, field: &Field) {
    let in_corner_left = pos.x < CORNER_RADIUS;
    let in_corner_right = pos.x > field.width - CORNER_RADIUS;
    let in_corner_top = pos.y < CORNER_RADIUS;
    let in_corner_bottom = pos.y > field.height - CORNER_RADIUS;

    // Left wall (only if not in corner region)
    if !in_corner_top && !in_corner_bottom && pos.x - radius < 0.0 {
        pos.x = radius;
        vel.x = vel.x.abs();
    }
    // Right wall
    if !in_corner_top && !in_corner_bottom && pos.x + radius > field.width {
        pos.x = field.width - radius;
        vel.x = -vel.x.abs();
    }
    // Top wall
    if !in_corner_left && !in_corner_right && pos.y - radius < 0.0 {
        pos.y = radius;
        vel.y = vel.y.abs();
    }
    // Bottom wall
    if !in_corner_left && !in_corner_right && pos.y + radius > field.height {
        pos.y = field.height - radius;
        vel.y = -vel.y.abs();
    }

    // Corner collisions
    collide_circle_corner(pos, vel, radius, field);
}

pub fn collide_penguin_wall(penguin: &mut Penguin, field: &Field) {
    collide_circle_wall(&mut penguin.pos, &mut penguin.vel, penguin.radius, field);
}

pub fn collide_ball_wall(ball: &mut Ball, field: &Field) {
    let in_corner_left = ball.pos.x < CORNER_RADIUS;
    let in_corner_right = ball.pos.x > field.width - CORNER_RADIUS;

    // Top wall (skip corner regions)
    if !in_corner_left && !in_corner_right && ball.pos.y - ball.radius < 0.0 {
        ball.pos.y = ball.radius;
        ball.vel.y = ball.vel.y.abs();
    }
    // Bottom wall (skip corner regions)
    if !in_corner_left && !in_corner_right && ball.pos.y + ball.radius > field.height {
        ball.pos.y = field.height - ball.radius;
        ball.vel.y = -ball.vel.y.abs();
    }

    // Corner collisions (before goal-related left/right checks)
    collide_circle_corner(&mut ball.pos, &mut ball.vel, ball.radius, field);

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
