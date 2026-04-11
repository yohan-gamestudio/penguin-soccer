use std::collections::HashMap;

use crate::domain::action::Action;
use crate::domain::ball::Ball;
use crate::domain::field::Field;
use crate::domain::game_state::{GameState, TurnPhase};
use crate::domain::penguin::Penguin;
use crate::domain::types::{PlayerId, TeamId, Vec2};
use crate::physics::{collision, movement};
use crate::ports::event::GameEvent;
use crate::rules::{match_flow, scoring, turn};

pub struct GameEngine {
    pub state: GameState,
    actions: HashMap<PlayerId, Action>,
    player_ids: Vec<PlayerId>,
    initial_positions: Vec<(PlayerId, Vec2)>,
    initial_ball_pos: Vec2,
}

impl GameEngine {
    pub fn new(field: Field, players: Vec<(PlayerId, TeamId, Vec2)>, ball_pos: Vec2, match_duration: f64) -> Self {
        let mut penguins = Vec::new();
        let mut player_ids = Vec::new();
        let mut initial_positions = Vec::new();
        let mut scores = HashMap::new();

        scores.insert(0, 0u32);
        scores.insert(1, 0u32);

        for &(id, team, pos) in &players {
            penguins.push(Penguin::new(id, team, pos));
            player_ids.push(id);
            initial_positions.push((id, pos));
        }

        let ball = Ball::new(ball_pos);

        let state = GameState {
            penguins,
            ball,
            field,
            scores,
            match_timer: match_duration,
            phase: TurnPhase::new_planning(turn::PLANNING_DEADLINE),
        };

        Self {
            state,
            actions: HashMap::new(),
            player_ids,
            initial_positions,
            initial_ball_pos: ball_pos,
        }
    }

    pub fn submit_action(&mut self, player_id: PlayerId, action: Action) -> Result<bool, String> {
        turn::submit_action(
            &mut self.state.phase,
            &mut self.actions,
            player_id,
            action,
            self.player_ids.len(),
        )
        .map_err(|e| format!("{:?}", e))
    }

    pub fn tick(&mut self, dt: f64) -> Vec<GameEvent> {
        let mut events = Vec::new();

        match &self.state.phase {
            TurnPhase::Planning { .. } => {
                let timed_out = turn::check_timeout(
                    &mut self.state.phase,
                    dt,
                    &self.player_ids,
                    &mut self.actions,
                );
                let all_submitted = self.actions.len() >= self.player_ids.len();
                if timed_out || all_submitted {
                    self.start_simulation(&mut events);
                }
            }
            TurnPhase::Simulating => {
                self.simulate_step(dt);
                self.state.match_timer -= dt;

                if match_flow::check_match_end(self.state.match_timer) {
                    self.state.match_timer = 0.0;
                    let result = match_flow::determine_winner(&self.state.scores);
                    events.push(GameEvent::MatchEnded(result));
                    self.state.phase = TurnPhase::Ended;
                    return events;
                }

                if let Some(scoring_team) = scoring::check_goal(&self.state.ball, &self.state.field) {
                    *self.state.scores.entry(scoring_team).or_insert(0) += 1;
                    events.push(GameEvent::GoalScored { scoring_team });
                    self.reset_positions();
                    self.state.phase = TurnPhase::new_planning(turn::PLANNING_DEADLINE);
                    events.push(GameEvent::PhaseChanged(self.state.phase.clone()));
                } else if movement::all_stopped(&self.state.penguins, &self.state.ball) {
                    self.state.phase = TurnPhase::Resolving;
                    events.push(GameEvent::PhaseChanged(self.state.phase.clone()));
                }
            }
            TurnPhase::Resolving => {
                self.state.phase = TurnPhase::new_planning(turn::PLANNING_DEADLINE);
                self.actions.clear();
                events.push(GameEvent::PhaseChanged(self.state.phase.clone()));
            }
            TurnPhase::Ended => {}
        }

        events
    }

    fn start_simulation(&mut self, events: &mut Vec<GameEvent>) {
        // Apply actions as velocities
        for penguin in &mut self.state.penguins {
            if let Some(action) = self.actions.get(&penguin.id) {
                penguin.vel = action.to_velocity();
            }
        }
        self.state.phase = TurnPhase::Simulating;
        events.push(GameEvent::PhaseChanged(self.state.phase.clone()));
    }

    fn simulate_step(&mut self, dt: f64) {
        // Move all objects
        for penguin in &mut self.state.penguins {
            movement::apply_movement_penguin(penguin, dt);
        }
        movement::apply_movement_ball(&mut self.state.ball, dt);

        // Wall collisions
        for penguin in &mut self.state.penguins {
            collision::collide_penguin_wall(penguin, &self.state.field);
        }
        collision::collide_ball_wall(&mut self.state.ball, &self.state.field);

        // Penguin-penguin collisions
        let len = self.state.penguins.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (left, right) = self.state.penguins.split_at_mut(j);
                collision::collide_penguin_penguin(&mut left[i], &mut right[0]);
            }
        }

        // Penguin-ball collisions
        for penguin in &mut self.state.penguins {
            collision::collide_penguin_ball(penguin, &mut self.state.ball);
        }
    }

    fn reset_positions(&mut self) {
        for penguin in &mut self.state.penguins {
            if let Some(&(_, pos)) = self.initial_positions.iter().find(|&&(id, _)| id == penguin.id) {
                penguin.pos = pos;
            }
            penguin.vel = Vec2::ZERO;
        }
        self.state.ball.pos = self.initial_ball_pos;
        self.state.ball.vel = Vec2::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::field::Field;
    use crate::domain::types::Vec2;

    fn create_test_engine() -> GameEngine {
        let field = Field::new(800.0, 400.0);
        let players = vec![
            (0, 0, Vec2::new(200.0, 200.0)),
            (1, 1, Vec2::new(600.0, 200.0)),
        ];
        let ball_pos = Vec2::new(400.0, 200.0);
        GameEngine::new(field, players, ball_pos, 120.0)
    }

    #[test]
    fn test_goal_resets_velocities() {
        let mut engine = create_test_engine();

        // Place ball at left goal with velocity
        engine.state.ball.pos = Vec2::new(5.0, 200.0);
        engine.state.ball.vel = Vec2::new(-100.0, 50.0);
        engine.state.penguins[0].vel = Vec2::new(50.0, -30.0);
        engine.state.penguins[1].vel = Vec2::new(-20.0, 10.0);
        engine.state.phase = TurnPhase::Simulating;

        let events = engine.tick(1.0 / 60.0);

        let goal_scored = events.iter().any(|e| matches!(e, GameEvent::GoalScored { .. }));
        assert!(goal_scored, "Goal should have been scored");

        assert_eq!(engine.state.ball.vel, Vec2::ZERO, "Ball velocity should be zero after goal");
        for penguin in &engine.state.penguins {
            assert_eq!(penguin.vel, Vec2::ZERO, "Penguin velocity should be zero after goal");
        }
    }

    #[test]
    fn test_goal_resets_positions() {
        let mut engine = create_test_engine();

        engine.state.ball.pos = Vec2::new(5.0, 200.0);
        engine.state.ball.vel = Vec2::new(-100.0, 0.0);
        engine.state.penguins[0].pos = Vec2::new(100.0, 100.0);
        engine.state.penguins[1].pos = Vec2::new(700.0, 300.0);
        engine.state.phase = TurnPhase::Simulating;

        engine.tick(1.0 / 60.0);

        assert_eq!(engine.state.ball.pos, Vec2::new(400.0, 200.0), "Ball should reset to center");
        assert_eq!(engine.state.penguins[0].pos, Vec2::new(200.0, 200.0), "Penguin 0 should reset");
        assert_eq!(engine.state.penguins[1].pos, Vec2::new(600.0, 200.0), "Penguin 1 should reset");
    }

    #[test]
    fn test_match_ended_only_fires_once() {
        let mut engine = create_test_engine();
        engine.state.match_timer = 0.01; // Almost over
        engine.state.phase = TurnPhase::Simulating;

        let events1 = engine.tick(1.0 / 60.0);
        let match_ended_count1 = events1.iter().filter(|e| matches!(e, GameEvent::MatchEnded(_))).count();
        assert_eq!(match_ended_count1, 1, "MatchEnded should fire once");

        // Second tick should NOT fire MatchEnded again
        let events2 = engine.tick(1.0 / 60.0);
        let match_ended_count2 = events2.iter().filter(|e| matches!(e, GameEvent::MatchEnded(_))).count();
        assert_eq!(match_ended_count2, 0, "MatchEnded should NOT fire again");
    }

    #[test]
    fn test_phase_transitions_to_planning_after_goal() {
        let mut engine = create_test_engine();
        engine.state.ball.pos = Vec2::new(5.0, 200.0);
        engine.state.ball.vel = Vec2::new(-100.0, 0.0);
        engine.state.phase = TurnPhase::Simulating;

        engine.tick(1.0 / 60.0);

        assert!(
            matches!(engine.state.phase, TurnPhase::Planning { .. }),
            "Phase should transition to Planning after goal"
        );
    }
}
