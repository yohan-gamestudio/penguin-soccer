use penguin_soccer_engine::domain::action::{Action, MAX_POWER};
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::GameEvent;
use penguin_soccer_engine::rules::turn::PLANNING_DEADLINE;

const DT: f64 = 1.0 / 60.0;

fn make_1v1() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0, 0, Vec2::new(200.0, 200.0)),
        (1, 1, Vec2::new(600.0, 200.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0)
}

/// Transition engine from Planning to Simulating by submitting both actions
/// then ticking past the deadline.
fn enter_simulating(engine: &mut GameEngine) {
    let action = Action::new(Vec2::new(1.0, 0.0), 0.0); // idle-ish, just to submit
    engine.submit_action(0, action).unwrap();
    engine.submit_action(1, action).unwrap();
    engine.tick(PLANNING_DEADLINE + 1.0);
    assert!(
        matches!(engine.state.phase, TurnPhase::Simulating),
        "expected Simulating after deadline tick"
    );
}

/// Run simulation ticks until the phase changes away from Simulating.
/// Returns the events from the tick that caused the transition.
fn run_until_phase_change(engine: &mut GameEngine) -> Vec<GameEvent> {
    for _ in 0..10_000 {
        let events = engine.tick(DT);
        if !matches!(engine.state.phase, TurnPhase::Simulating) {
            return events;
        }
        // also return if events contain a phase-change
        if events
            .iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(_)))
        {
            return events;
        }
    }
    panic!("simulation did not stop within 10 000 ticks");
}

// ── Test 1 ──────────────────────────────────────────────────────────────────
// Simulating phase: when all objects stop, transitions to Resolving.

#[test]
fn simulating_all_stopped_transitions_to_resolving() {
    let mut engine = make_1v1();
    // Both players submit idle actions → no velocity → objects already stopped.
    enter_simulating(&mut engine);

    let events = run_until_phase_change(&mut engine);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Resolving))),
        "expected PhaseChanged(Resolving) event"
    );
    assert!(
        matches!(engine.state.phase, TurnPhase::Resolving),
        "phase must be Resolving after all stopped"
    );
}

// ── Test 2 ──────────────────────────────────────────────────────────────────
// Resolving → tick → transitions back to Planning.

#[test]
fn resolving_transitions_back_to_planning() {
    let mut engine = make_1v1();
    enter_simulating(&mut engine);
    run_until_phase_change(&mut engine); // → Resolving
    assert!(matches!(engine.state.phase, TurnPhase::Resolving));

    let events = engine.tick(DT);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Planning { .. }))),
        "expected PhaseChanged(Planning) event from Resolving"
    );
    assert!(
        matches!(engine.state.phase, TurnPhase::Planning { .. }),
        "phase must be Planning after Resolving tick"
    );
}

// ── Test 3 ──────────────────────────────────────────────────────────────────
// Action power=1.0 → velocity = direction * MAX_POWER (500).

#[test]
fn action_power_1_converts_to_max_velocity() {
    let dir = Vec2::new(1.0, 0.0);
    let action = Action::new(dir, 1.0);
    let vel = action.to_velocity();

    let expected = MAX_POWER; // 500.0
    assert!(
        (vel.x - expected).abs() < 1e-9,
        "vel.x should be {expected}, got {}",
        vel.x
    );
    assert!(
        vel.y.abs() < 1e-9,
        "vel.y should be 0, got {}",
        vel.y
    );
}

// ── Test 4 ──────────────────────────────────────────────────────────────────
// Action power=0.5 → velocity = direction * 250.

#[test]
fn action_power_half_converts_to_half_velocity() {
    let dir = Vec2::new(1.0, 0.0);
    let action = Action::new(dir, 0.5);
    let vel = action.to_velocity();

    let expected = MAX_POWER * 0.5; // 250.0
    assert!(
        (vel.x - expected).abs() < 1e-9,
        "vel.x should be {expected}, got {}",
        vel.x
    );
    assert!(
        vel.y.abs() < 1e-9,
        "vel.y should be 0, got {}",
        vel.y
    );
}

// ── Test 5 ──────────────────────────────────────────────────────────────────
// Multiple penguins fire simultaneously and move.

#[test]
fn multiple_penguins_fire_and_move() {
    let mut engine = make_1v1();

    let initial_pos_0 = engine.state.penguins[0].pos;
    let initial_pos_1 = engine.state.penguins[1].pos;

    // Player 0 fires right, player 1 fires left.
    let action_0 = Action::new(Vec2::new(1.0, 0.0), 1.0);
    let action_1 = Action::new(Vec2::new(-1.0, 0.0), 1.0);
    engine.submit_action(0, action_0).unwrap();
    engine.submit_action(1, action_1).unwrap();
    engine.tick(PLANNING_DEADLINE + 1.0); // → Simulating, velocities applied

    // One simulation step
    engine.tick(DT);

    let pos_0 = engine.state.penguins[0].pos;
    let pos_1 = engine.state.penguins[1].pos;

    assert!(
        pos_0.x > initial_pos_0.x,
        "penguin 0 should have moved right: {:.2} -> {:.2}",
        initial_pos_0.x,
        pos_0.x
    );
    assert!(
        pos_1.x < initial_pos_1.x,
        "penguin 1 should have moved left: {:.2} -> {:.2}",
        initial_pos_1.x,
        pos_1.x
    );
}

// ── Test 6 ──────────────────────────────────────────────────────────────────
// Full simulation: fire penguin toward ball → ball moves → everything stops → Resolving.

#[test]
fn fire_penguin_at_ball_ball_moves_then_resolves() {
    let mut engine = make_1v1();

    // Player 0 is at (200, 200), ball at (400, 200). Fire right toward ball.
    let action_0 = Action::new(Vec2::new(1.0, 0.0), 1.0);
    // Player 1 idles.
    let action_1 = Action::idle();

    engine.submit_action(0, action_0).unwrap();
    engine.submit_action(1, action_1).unwrap();
    engine.tick(PLANNING_DEADLINE + 1.0); // → Simulating

    // Record ball position just after simulation starts.
    let ball_start = engine.state.ball.pos;

    // Run until the phase changes (Resolving or Planning via goal).
    let events = run_until_phase_change(&mut engine);

    let ball_end = engine.state.ball.pos;

    // Ball must have moved from its initial position (hit by penguin).
    assert!(
        (ball_end.x - ball_start.x).abs() > 1.0 || (ball_end.y - ball_start.y).abs() > 1.0,
        "ball should have moved after penguin collision; start={:?} end={:?}",
        ball_start,
        ball_end
    );

    // Phase should have transitioned to Resolving (or Planning if a goal occurred).
    let phase_changed = events.iter().any(|e| matches!(e, GameEvent::PhaseChanged(_)));
    assert!(
        phase_changed,
        "expected a PhaseChanged event after simulation ends"
    );
    let not_simulating = !matches!(engine.state.phase, TurnPhase::Simulating);
    assert!(not_simulating, "phase must not be Simulating after all stop");
}
