use crate::domain::ball::Ball;
use crate::domain::field::Field;
use crate::domain::types::TeamId;

pub fn check_goal(ball: &Ball, field: &Field) -> Option<TeamId> {
    for goal in &field.goals {
        let goal_half = goal.width / 2.0;
        let goal_top = goal.center.y - goal_half;
        let goal_bottom = goal.center.y + goal_half;

        let in_y = ball.pos.y >= goal_top && ball.pos.y <= goal_bottom;

        // Left goal (team 0 defends): ball crosses x=0
        if goal.team == 0 && ball.pos.x - ball.radius <= 0.0 && in_y {
            return Some(1); // Team 1 scores
        }
        // Right goal (team 1 defends): ball crosses x=width
        if goal.team == 1 && ball.pos.x + ball.radius >= field.width && in_y {
            return Some(0); // Team 0 scores
        }
    }
    None
}
