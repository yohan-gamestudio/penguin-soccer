use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::{GameEvent, GameEventPort};
use penguin_soccer_engine::rules::match_flow::MatchResult;
use penguin_soccer_engine::rules::turn::PLANNING_DEADLINE;

const DT: f64 = 1.0 / 60.0;

// ── Mock implementation of GameEventPort ────────────────────────────────────

struct MockEventCollector {
    events: Vec<GameEvent>,
}

impl MockEventCollector {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl GameEventPort for MockEventCollector {
    fn on_event(&mut self, event: &GameEvent) {
        self.events.push(event.clone());
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_1v1() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0)
}

/// Tick until a PhaseChanged event appears, collecting all events along the way.
fn run_until_phase_change(engine: &mut GameEngine) -> Vec<GameEvent> {
    for _ in 0..10_000 {
        let evts = engine.tick(DT);
        let has_phase = evts
            .iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(_)));
        if has_phase {
            return evts;
        }
    }
    vec![]
}

// ── Test 1: MockEventCollector implements GameEventPort ──────────────────────

#[test]
fn mock_event_collector_implements_port() {
    let mut collector = MockEventCollector::new();
    assert!(collector.events.is_empty());

    // Drive a port through our mock by hand
    let event = GameEvent::GoalScored { scoring_team: 0 };
    collector.on_event(&event);

    assert_eq!(collector.events.len(), 1);
    assert!(matches!(
        &collector.events[0],
        GameEvent::GoalScored { scoring_team: 0 }
    ));
}

// ── Test 2: tick() returns PhaseChanged(Simulating) on transition ────────────

#[test]
fn tick_returns_phase_changed_when_transitioning_to_simulating() {
    let mut engine = make_1v1();

    // Force timeout → Planning → Simulating
    let events = engine.tick(PLANNING_DEADLINE + 1.0);

    let has_simulating = events
        .iter()
        .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Simulating)));
    assert!(
        has_simulating,
        "tick should emit PhaseChanged(Simulating) on timeout"
    );
}

// ── Test 3: GoalScored contains correct team id ──────────────────────────────

#[test]
fn goal_scored_event_contains_correct_team_id() {
    let field = Field::new(800.0, 400.0);
    let goal_y = 200.0; // center of field height

    // Place the ball just inside team-1's goal (right side, x = width)
    // Team 0 scores when ball crosses x = field.width
    let ball_pos = Vec2::new(790.0, goal_y);

    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let mut engine = GameEngine::new(field, players, ball_pos, 120.0);

    // Move the ball toward the right goal with enough velocity to cross
    engine.state.ball.vel = Vec2::new(2000.0, 0.0);
    engine.state.phase = TurnPhase::Simulating;

    // Tick until GoalScored appears
    let mut goal_event: Option<GameEvent> = None;
    for _ in 0..1000 {
        let evts = engine.tick(DT);
        if let Some(e) = evts
            .iter()
            .find(|e| matches!(e, GameEvent::GoalScored { .. }))
        {
            goal_event = Some(e.clone());
            break;
        }
    }

    let event = goal_event.expect("GoalScored event should have fired");
    match event {
        GameEvent::GoalScored { scoring_team } => {
            assert_eq!(
                scoring_team, 0,
                "Team 0 should score when ball enters right goal"
            );
        }
        _ => panic!("Expected GoalScored event"),
    }
}

// ── Test 4: MatchEnded contains correct MatchResult ──────────────────────────

#[test]
fn match_ended_event_contains_correct_result_draw() {
    // Very short match with no goals → Draw.
    // Use a large dt so the timer expires in a single Simulating tick before
    // all_stopped() can fire and transition us away from Simulating.
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let short_duration = 1.0;
    let mut engine = GameEngine::new(field, players, Vec2::new(400.0, 200.0), short_duration);

    // Force into simulating; use a dt larger than the duration so the match
    // ends on the very first Simulating tick.
    engine.state.phase = TurnPhase::Simulating;
    let evts = engine.tick(short_duration + 1.0);

    let ended_event = evts
        .iter()
        .find(|e| matches!(e, GameEvent::MatchEnded(_)))
        .cloned()
        .expect("MatchEnded event should have fired");

    match ended_event {
        GameEvent::MatchEnded(result) => {
            assert_eq!(result, MatchResult::Draw, "No goals → should be a Draw");
        }
        _ => panic!("Expected MatchEnded event"),
    }
}

#[test]
fn match_ended_event_contains_correct_result_win() {
    // Team 0 scores one goal, then we drain the timer in a single large tick.
    let field = Field::new(800.0, 400.0);
    let goal_y = 200.0;
    let ball_pos = Vec2::new(790.0, goal_y);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let mut engine = GameEngine::new(field, players, ball_pos, 120.0);

    // Score a goal for team 0 first
    engine.state.ball.vel = Vec2::new(3000.0, 0.0);
    engine.state.phase = TurnPhase::Simulating;

    let mut goal_fired = false;
    for _ in 0..2000 {
        let evts = engine.tick(DT);
        if evts
            .iter()
            .any(|e| matches!(e, GameEvent::GoalScored { .. }))
        {
            goal_fired = true;
            break;
        }
    }
    assert!(goal_fired, "setup: goal should have fired");
    assert_eq!(*engine.state.scores.get(&0u8).unwrap_or(&0), 1);

    // Now drain the timer in one shot from whichever phase we're in
    engine.state.phase = TurnPhase::Simulating;
    let evts = engine.tick(engine.state.match_timer + 1.0);

    let result = evts
        .iter()
        .find_map(|e| {
            if let GameEvent::MatchEnded(r) = e {
                Some(r.clone())
            } else {
                None
            }
        })
        .expect("MatchEnded should fire");

    match result {
        MatchResult::Win(team) => assert_eq!(team, 0, "Team 0 scored → should win"),
        MatchResult::Draw => panic!("Expected Win(0), got Draw"),
    }
}

// ── Test 5: Event sequence Planning→Simulating→Resolving→Planning ───────────

#[test]
fn event_sequence_full_turn_via_port() {
    let mut engine = make_1v1();
    let mut collector = MockEventCollector::new();

    // Planning → Simulating (via timeout)
    let evts = engine.tick(PLANNING_DEADLINE + 1.0);
    for e in &evts {
        collector.on_event(e);
    }
    assert!(
        matches!(engine.state.phase, TurnPhase::Simulating),
        "should be Simulating after timeout"
    );

    // Simulating → Resolving (idle actions → already stopped)
    let evts = run_until_phase_change(&mut engine);
    for e in &evts {
        collector.on_event(e);
    }
    assert!(
        matches!(engine.state.phase, TurnPhase::Resolving),
        "should be Resolving after all stopped"
    );

    // Resolving → Planning
    let evts = engine.tick(DT);
    for e in &evts {
        collector.on_event(e);
    }
    assert!(
        matches!(engine.state.phase, TurnPhase::Planning { .. }),
        "should be back to Planning"
    );

    // Verify collected sequence
    let phases: Vec<_> = collector
        .events
        .iter()
        .filter_map(|e| {
            if let GameEvent::PhaseChanged(p) = e {
                Some(p.clone())
            } else {
                None
            }
        })
        .collect();

    assert!(phases.len() >= 3, "Should have at least 3 PhaseChanged events");
    assert!(
        matches!(phases[0], TurnPhase::Simulating),
        "First transition should be to Simulating"
    );
    assert!(
        matches!(phases[1], TurnPhase::Resolving),
        "Second transition should be to Resolving"
    );
    assert!(
        matches!(phases[2], TurnPhase::Planning { .. }),
        "Third transition should be back to Planning"
    );
}
