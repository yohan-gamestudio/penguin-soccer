use penguin_soccer_engine::domain::ball::Ball;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::rules::scoring;

// Field::new(800, 400):
//   goal_width = 400 * 0.3 = 120
//   goal_y     = 400 / 2.0 = 200
//   left goal  center = (0, 200),   team=0, y range [140, 260]
//   right goal center = (800, 200), team=1, y range [140, 260]
// Ball radius = 10.0

fn make_field() -> Field {
    Field::new(800.0, 400.0)
}

fn ball_at(x: f64, y: f64) -> Ball {
    Ball::new(Vec2::new(x, y))
}

// ── unit tests: check_goal ────────────────────────────────────────────────────

#[test]
fn ball_enters_left_goal_team1_scores() {
    let field = make_field();
    // ball.pos.x - radius <= 0  →  pos.x <= radius (10.0)
    let ball = ball_at(5.0, 200.0); // center of goal, just inside left line
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(1), "team 1 should score when ball enters left goal");
}

#[test]
fn ball_enters_right_goal_team0_scores() {
    let field = make_field();
    // ball.pos.x + radius >= 800  →  pos.x >= 790.0
    let ball = ball_at(795.0, 200.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(0), "team 0 should score when ball enters right goal");
}

#[test]
fn ball_at_left_goal_line_exactly_scores() {
    let field = make_field();
    // pos.x - radius = 0  →  pos.x = 10.0 (radius)
    let ball = ball_at(10.0, 200.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(1));
}

#[test]
fn ball_at_right_goal_line_exactly_scores() {
    let field = make_field();
    // pos.x + radius = 800  →  pos.x = 790.0
    let ball = ball_at(790.0, 200.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(0));
}

#[test]
fn ball_near_left_goal_outside_y_range_no_goal() {
    let field = make_field();
    // y = 130.0 is above goal top (140.0)
    let ball = ball_at(5.0, 130.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, None, "ball above left goal y range should not score");
}

#[test]
fn ball_near_left_goal_below_y_range_no_goal() {
    let field = make_field();
    // y = 270.0 is below goal bottom (260.0)
    let ball = ball_at(5.0, 270.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, None, "ball below left goal y range should not score");
}

#[test]
fn ball_near_right_goal_outside_y_range_no_goal() {
    let field = make_field();
    let ball = ball_at(795.0, 130.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, None, "ball above right goal y range should not score");
}

#[test]
fn ball_near_right_goal_below_y_range_no_goal() {
    let field = make_field();
    let ball = ball_at(795.0, 270.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, None, "ball below right goal y range should not score");
}

#[test]
fn ball_in_middle_of_field_no_goal() {
    let field = make_field();
    let ball = ball_at(400.0, 200.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, None, "ball in center of field should not score");
}

#[test]
fn ball_at_goal_y_boundary_top_inclusive() {
    let field = make_field();
    // goal_top = 200 - 60 = 140.0
    let ball = ball_at(5.0, 140.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(1), "ball exactly at goal top boundary should score");
}

#[test]
fn ball_at_goal_y_boundary_bottom_inclusive() {
    let field = make_field();
    // goal_bottom = 200 + 60 = 260.0
    let ball = ball_at(5.0, 260.0);
    let result = scoring::check_goal(&ball, &field);
    assert_eq!(result, Some(1), "ball exactly at goal bottom boundary should score");
}

// ── integration tests: GameEngine ────────────────────────────────────────────

fn make_engine_with_ball_at(ball_x: f64, ball_y: f64) -> GameEngine {
    let field = make_field();
    // Two players, one per team, placed well away from goals
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    GameEngine::new(field, players, Vec2::new(ball_x, ball_y), 300.0)
}

/// Force the engine into Simulating phase by ticking past the planning deadline.
/// The planning deadline is turn::PLANNING_DEADLINE seconds; tick with a large dt.
fn skip_to_simulating(engine: &mut GameEngine) {
    use penguin_soccer_engine::domain::game_state::TurnPhase;
    // tick with a big dt to time out the planning phase
    let big_dt = 100.0;
    engine.tick(big_dt);
    // should now be Simulating
    assert!(
        matches!(engine.state.phase, TurnPhase::Simulating),
        "expected Simulating phase after planning timeout"
    );
}

#[test]
fn integration_goal_event_emitted_and_score_incremented() {
    use penguin_soccer_engine::ports::event::GameEvent;
    use penguin_soccer_engine::domain::game_state::TurnPhase;

    // Ball starts just inside left goal, in y range
    let mut engine = make_engine_with_ball_at(5.0, 200.0);
    skip_to_simulating(&mut engine);

    // One simulate tick — ball is already in goal, goal should be detected
    let events = engine.tick(1.0 / 60.0);

    let goal_events: Vec<_> = events
        .iter()
        .filter_map(|e| if let GameEvent::GoalScored { scoring_team } = e { Some(*scoring_team) } else { None })
        .collect();

    assert_eq!(goal_events, vec![1], "team 1 should score entering the left goal");
    assert_eq!(*engine.state.scores.get(&1).unwrap(), 1, "score for team 1 should be 1");
    assert_eq!(*engine.state.scores.get(&0).unwrap(), 0, "score for team 0 should be 0");

    // Phase should have reset to Planning
    assert!(
        matches!(engine.state.phase, TurnPhase::Planning { .. }),
        "phase should reset to Planning after goal"
    );
}

#[test]
fn integration_positions_reset_after_goal() {
    let initial_ball = Vec2::new(400.0, 200.0);
    let player0_initial = Vec2::new(200.0, 200.0);
    let player1_initial = Vec2::new(600.0, 200.0);

    // Engine constructed with ball at center — that becomes initial_ball_pos for reset
    let mut engine = make_engine_with_ball_at(400.0, 200.0);
    skip_to_simulating(&mut engine);

    // Move penguins away from initial spots to confirm reset
    engine.state.penguins[0].pos = Vec2::new(50.0, 50.0);
    engine.state.penguins[1].pos = Vec2::new(750.0, 350.0);
    // Place ball inside the left goal for this tick
    engine.state.ball.pos = Vec2::new(5.0, 200.0);
    engine.state.ball.vel = Vec2::ZERO;

    engine.tick(1.0 / 60.0); // simulate_step (ball stays ~5,200 with zero vel) → goal detected → reset

    // Ball back to initial (center)
    assert_eq!(engine.state.ball.pos, initial_ball, "ball should reset to initial position");

    // Penguins back to initial
    let p0 = engine.state.penguins.iter().find(|p| p.id == 0).unwrap();
    let p1 = engine.state.penguins.iter().find(|p| p.id == 1).unwrap();
    assert_eq!(p0.pos, player0_initial, "player 0 should reset to initial position");
    assert_eq!(p1.pos, player1_initial, "player 1 should reset to initial position");

    // Velocities zeroed
    assert_eq!(p0.vel, Vec2::ZERO, "player 0 velocity should be zero after reset");
    assert_eq!(p1.vel, Vec2::ZERO, "player 1 velocity should be zero after reset");
    assert_eq!(engine.state.ball.vel, Vec2::ZERO, "ball velocity should be zero after reset");
}

#[test]
fn integration_score_increments_on_multiple_goals() {
    use penguin_soccer_engine::domain::game_state::TurnPhase;

    let mut engine = make_engine_with_ball_at(5.0, 200.0);

    for expected_score in 1u32..=3 {
        // Enter simulating phase
        skip_to_simulating(&mut engine);

        // Place ball inside left goal
        engine.state.ball.pos = Vec2::new(5.0, 200.0);
        engine.state.ball.vel = Vec2::ZERO;

        engine.tick(1.0 / 60.0); // triggers goal

        assert_eq!(
            *engine.state.scores.get(&1).unwrap(),
            expected_score,
            "team 1 score should be {expected_score} after goal #{expected_score}"
        );

        // Confirm we're back in Planning so the next loop iteration can skip to Simulating
        assert!(
            matches!(engine.state.phase, TurnPhase::Planning { .. }),
            "should be in Planning after goal #{expected_score}"
        );
    }

    assert_eq!(*engine.state.scores.get(&0).unwrap(), 0, "team 0 should never have scored");
}

#[test]
fn integration_right_goal_team0_scores() {
    use penguin_soccer_engine::ports::event::GameEvent;

    let mut engine = make_engine_with_ball_at(795.0, 200.0);
    skip_to_simulating(&mut engine);

    let events = engine.tick(1.0 / 60.0);

    let goal_events: Vec<_> = events
        .iter()
        .filter_map(|e| if let GameEvent::GoalScored { scoring_team } = e { Some(*scoring_team) } else { None })
        .collect();

    assert_eq!(goal_events, vec![0], "team 0 should score entering the right goal");
    assert_eq!(*engine.state.scores.get(&0).unwrap(), 1);
    assert_eq!(*engine.state.scores.get(&1).unwrap(), 0);
}
