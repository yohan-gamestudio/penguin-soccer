use std::collections::HashMap;

use penguin_soccer_engine::domain::action::Action;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::GameEvent;
use penguin_soccer_engine::rules::match_flow::{determine_winner, MatchResult};
use penguin_soccer_engine::rules::turn::PLANNING_DEADLINE;

const DT: f64 = 1.0 / 60.0;

fn make_engine(match_duration: f64) -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0, 0, Vec2::new(200.0, 200.0)),
        (1, 1, Vec2::new(600.0, 200.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), match_duration)
}

/// Submit idle actions for both players then tick past planning deadline → Simulating.
fn enter_simulating(engine: &mut GameEngine) {
    engine.submit_action(0, Action::idle()).unwrap();
    engine.submit_action(1, Action::idle()).unwrap();
    engine.tick(PLANNING_DEADLINE + 1.0);
    assert!(
        matches!(engine.state.phase, TurnPhase::Simulating),
        "expected Simulating"
    );
}

// ── Test 1 ──────────────────────────────────────────────────────────────────
// Game starts with timer at the specified duration and scores 0:0.

#[test]
fn game_starts_with_correct_timer_and_zero_scores() {
    let duration = 120.0;
    let engine = make_engine(duration);

    assert!(
        (engine.state.match_timer - duration).abs() < 1e-9,
        "match_timer should start at {duration}, got {}",
        engine.state.match_timer
    );
    assert_eq!(
        engine.state.scores.get(&0).copied().unwrap_or(0),
        0,
        "team 0 score should start at 0"
    );
    assert_eq!(
        engine.state.scores.get(&1).copied().unwrap_or(0),
        0,
        "team 1 score should start at 0"
    );
}

// ── Test 2 ──────────────────────────────────────────────────────────────────
// Timer decreases only during Simulating phase.

#[test]
fn timer_decreases_during_simulating() {
    let mut engine = make_engine(120.0);
    enter_simulating(&mut engine);

    let timer_before = engine.state.match_timer;
    engine.tick(DT);
    let timer_after = engine.state.match_timer;

    assert!(
        timer_after < timer_before,
        "match_timer should decrease during Simulating; before={timer_before} after={timer_after}"
    );
    assert!(
        (timer_before - timer_after - DT).abs() < 1e-9,
        "timer should decrease by exactly dt={DT}"
    );
}

// ── Test 3 ──────────────────────────────────────────────────────────────────
// Timer does NOT decrease during Planning phase.

#[test]
fn timer_does_not_decrease_during_planning() {
    let mut engine = make_engine(120.0);
    assert!(matches!(engine.state.phase, TurnPhase::Planning { .. }));

    let timer_before = engine.state.match_timer;
    // Tick several times while still in Planning (small dt, no timeout).
    for _ in 0..10 {
        engine.tick(DT);
    }
    let timer_after = engine.state.match_timer;

    assert!(
        (timer_after - timer_before).abs() < 1e-9,
        "match_timer must not change during Planning; before={timer_before} after={timer_after}"
    );
}

// ── Test 4 ──────────────────────────────────────────────────────────────────
// Timer reaches 0 → MatchEnded event is emitted.

#[test]
fn timer_reaching_zero_emits_match_ended() {
    // Very short match so a single Simulating tick crosses zero.
    let mut engine = make_engine(DT * 0.5);
    enter_simulating(&mut engine);

    // One tick that covers more than the remaining timer.
    let events = engine.tick(DT);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::MatchEnded(_))),
        "MatchEnded event should be emitted when timer reaches 0; events={events:?}"
    );
    assert!(
        (engine.state.match_timer - 0.0).abs() < 1e-9,
        "match_timer should be clamped to 0 after match ends"
    );
}

// ── Test 5 ──────────────────────────────────────────────────────────────────
// Higher-scoring team wins.

#[test]
fn higher_scoring_team_wins() {
    let mut scores = HashMap::new();
    scores.insert(0u8, 3u32);
    scores.insert(1u8, 1u32);

    assert_eq!(determine_winner(&scores), MatchResult::Win(0));

    let mut scores2 = HashMap::new();
    scores2.insert(0u8, 0u32);
    scores2.insert(1u8, 2u32);

    assert_eq!(determine_winner(&scores2), MatchResult::Win(1));
}

// ── Test 6 ──────────────────────────────────────────────────────────────────
// Equal scores → Draw.

#[test]
fn equal_scores_produce_draw() {
    let mut scores = HashMap::new();
    scores.insert(0u8, 2u32);
    scores.insert(1u8, 2u32);

    assert_eq!(determine_winner(&scores), MatchResult::Draw);
}

#[test]
fn zero_zero_is_draw() {
    let mut scores = HashMap::new();
    scores.insert(0u8, 0u32);
    scores.insert(1u8, 0u32);

    assert_eq!(determine_winner(&scores), MatchResult::Draw);
}

// ── Test 7 ──────────────────────────────────────────────────────────────────
// determine_winner unit tests: missing keys default to 0.

#[test]
fn determine_winner_missing_team_defaults_to_zero() {
    // Only team 0 present with score 1 → team 0 wins.
    let mut scores = HashMap::new();
    scores.insert(0u8, 1u32);

    assert_eq!(
        determine_winner(&scores),
        MatchResult::Win(0),
        "missing team 1 defaults to 0, team 0 with score 1 should win"
    );
}

#[test]
fn determine_winner_empty_scores_is_draw() {
    let scores: HashMap<u8, u32> = HashMap::new();
    assert_eq!(
        determine_winner(&scores),
        MatchResult::Draw,
        "empty scores map should be a Draw (both default to 0)"
    );
}

#[test]
fn determine_winner_team1_only_wins() {
    let mut scores = HashMap::new();
    scores.insert(1u8, 5u32);

    assert_eq!(
        determine_winner(&scores),
        MatchResult::Win(1),
        "only team 1 present with score 5 should win"
    );
}

// ── Integration: MatchEnded carries the correct result ────────────────────

#[test]
fn match_ended_event_carries_winner_result() {
    let mut engine = make_engine(DT * 0.5);

    // Artificially give team 1 a score advantage.
    *engine.state.scores.entry(1).or_insert(0) = 2;

    enter_simulating(&mut engine);
    let events = engine.tick(DT);

    let result = events.iter().find_map(|e| {
        if let GameEvent::MatchEnded(r) = e {
            Some(r.clone())
        } else {
            None
        }
    });

    assert!(result.is_some(), "MatchEnded event must be present");
    assert_eq!(
        result.unwrap(),
        MatchResult::Win(1),
        "team 1 should be the winner"
    );
}
