use penguin_soccer_engine::domain::action::Action;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::GameEvent;
use penguin_soccer_engine::rules::match_flow::MatchResult;
use penguin_soccer_engine::rules::turn::PLANNING_DEADLINE;

const DT: f64 = 1.0 / 60.0;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_1v1() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0)
}

fn make_2v2() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(150.0, 150.0)),
        (1u8, 0u8, Vec2::new(150.0, 250.0)),
        (2u8, 1u8, Vec2::new(650.0, 150.0)),
        (3u8, 1u8, Vec2::new(650.0, 250.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0)
}

/// 1v2: player 0 on team 0, players 1 and 2 on team 1
fn make_1v2() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 150.0)),
        (2u8, 1u8, Vec2::new(600.0, 250.0)),
    ];
    GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0)
}

/// Tick until a PhaseChanged event fires, return that tick's events.
/// Panics after 10 000 ticks without a phase change.
fn run_until_phase_change(engine: &mut GameEngine) -> Vec<GameEvent> {
    for _ in 0..10_000 {
        let evts = engine.tick(DT);
        let has_change = evts
            .iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(_)));
        if has_change {
            return evts;
        }
    }
    panic!("run_until_phase_change: no PhaseChanged after 10 000 ticks");
}

/// Force through Planning → Simulating → Resolving → Planning in one step each,
/// using idle actions and timeout.  Returns all collected events.
fn complete_one_idle_turn(engine: &mut GameEngine) -> Vec<GameEvent> {
    let mut all: Vec<GameEvent> = Vec::new();

    // Planning → Simulating via timeout
    let evts = engine.tick(PLANNING_DEADLINE + 1.0);
    all.extend(evts);

    // Simulating → Resolving (objects already stopped because actions were idle)
    let evts = run_until_phase_change(engine);
    all.extend(evts);

    // Resolving → Planning
    let evts = engine.tick(DT);
    all.extend(evts);

    all
}

fn is_planning(phase: &TurnPhase) -> bool {
    matches!(phase, TurnPhase::Planning { .. })
}

// ── Integration Test 1: Full 1-turn scenario ─────────────────────────────────

#[test]
fn full_turn_both_players_submit_then_cycle() {
    let mut engine = make_1v1();

    // Both players submit actions
    let action = Action::new(Vec2::new(1.0, 0.0), 0.3);
    engine.submit_action(0, action).expect("player 0 submit");
    engine.submit_action(1, action).expect("player 1 submit");

    // Timeout → Simulating
    let evts = engine.tick(PLANNING_DEADLINE + 1.0);
    assert!(
        evts.iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Simulating))),
        "should transition to Simulating"
    );
    assert!(matches!(engine.state.phase, TurnPhase::Simulating));

    // Simulating → Resolving (tick until stopped)
    let evts = run_until_phase_change(&mut engine);
    assert!(
        evts.iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Resolving))),
        "should transition to Resolving once stopped"
    );
    assert!(matches!(engine.state.phase, TurnPhase::Resolving));

    // Resolving → Planning
    let evts = engine.tick(DT);
    assert!(
        evts.iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Planning { .. }))),
        "should go back to Planning"
    );
    assert!(is_planning(&engine.state.phase));
}

// ── Integration Test 2: Multiple turns, timer decreases, match ends ───────────

#[test]
fn multiple_turns_timer_decreases_and_match_ends() {
    // Very short match so it ends quickly during simulation
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let mut engine = GameEngine::new(field, players, Vec2::new(400.0, 200.0), 0.5);

    let initial_timer = engine.state.match_timer;

    // Force into simulating phase directly so the timer ticks down
    engine.state.phase = TurnPhase::Simulating;

    let mut match_ended = false;
    let mut ticks = 0;
    for _ in 0..10_000 {
        let evts = engine.tick(DT);
        ticks += 1;
        if evts
            .iter()
            .any(|e| matches!(e, GameEvent::MatchEnded(_)))
        {
            match_ended = true;
            break;
        }
        // If we somehow land back in Planning keep simulating
        if matches!(engine.state.phase, TurnPhase::Planning { .. }) {
            engine.state.phase = TurnPhase::Simulating;
        }
    }

    assert!(
        engine.state.match_timer <= initial_timer,
        "match_timer should have decreased"
    );
    assert!(match_ended, "MatchEnded should fire after {} ticks", ticks);
    assert_eq!(engine.state.match_timer, 0.0, "timer should be clamped to 0");
}

// ── Integration Test 3: Goal scenario ────────────────────────────────────────

#[test]
fn goal_scenario_penguin_hits_ball_toward_goal_score_reset() {
    // Place penguin right next to ball, ball near right goal
    let field = Field::new(800.0, 400.0);
    let goal_y = 200.0;
    let ball_start = Vec2::new(790.0, goal_y);

    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let mut engine = GameEngine::new(field, players, ball_start, 120.0);

    // Directly push ball into the right goal
    engine.state.ball.vel = Vec2::new(3000.0, 0.0);
    engine.state.phase = TurnPhase::Simulating;

    let mut goal_fired = false;
    let mut post_goal_planning = false;

    for _ in 0..2000 {
        let evts = engine.tick(DT);

        for e in &evts {
            if let GameEvent::GoalScored { scoring_team } = e {
                goal_fired = true;
                assert_eq!(*scoring_team, 0, "Team 0 should score in right goal");
                assert_eq!(
                    *engine.state.scores.get(&0u8).unwrap_or(&0),
                    1,
                    "score should be 1 after goal"
                );
            }
            if goal_fired {
                if let GameEvent::PhaseChanged(TurnPhase::Planning { .. }) = e {
                    post_goal_planning = true;
                }
            }
        }

        if post_goal_planning {
            break;
        }
    }

    assert!(goal_fired, "GoalScored event should have fired");
    assert!(
        post_goal_planning,
        "should transition to Planning after goal"
    );

    // Positions should be reset to initial
    let p0 = engine.state.penguins.iter().find(|p| p.id == 0).unwrap();
    assert!(
        (p0.pos.x - 200.0).abs() < 1.0 && (p0.pos.y - 200.0).abs() < 1.0,
        "player 0 should be reset to initial position"
    );
    assert!(
        (engine.state.ball.pos.x - ball_start.x).abs() < 1.0,
        "ball should be reset to initial position"
    );
}

// ── Integration Test 4: 1v1 team composition ─────────────────────────────────

#[test]
fn composition_1v1_runs_full_turn() {
    let mut engine = make_1v1();
    assert_eq!(engine.state.penguins.len(), 2);

    complete_one_idle_turn(&mut engine);
    assert!(is_planning(&engine.state.phase));
}

// ── Integration Test 4: 2v2 team composition ─────────────────────────────────

#[test]
fn composition_2v2_runs_full_turn() {
    let mut engine = make_2v2();
    assert_eq!(engine.state.penguins.len(), 4);

    complete_one_idle_turn(&mut engine);
    assert!(is_planning(&engine.state.phase));
}

// ── Integration Test 4: 1v2 team composition ─────────────────────────────────

#[test]
fn composition_1v2_runs_full_turn() {
    let mut engine = make_1v2();
    assert_eq!(engine.state.penguins.len(), 3);

    complete_one_idle_turn(&mut engine);
    assert!(is_planning(&engine.state.phase));
}

// ── Integration Test 5: Asymmetric teams with explicit player list ────────────

#[test]
fn asymmetric_teams_explicit_player_list() {
    // Players: (player_id, team_id, pos)
    // player 0 → team 0; players 1 and 2 → team 1
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 150.0)),
        (2u8, 1u8, Vec2::new(600.0, 250.0)),
    ];
    let mut engine = GameEngine::new(field, players, Vec2::new(400.0, 200.0), 120.0);

    assert_eq!(engine.state.penguins.len(), 3);

    // Verify team assignments
    let p0 = engine.state.penguins.iter().find(|p| p.id == 0).unwrap();
    let p1 = engine.state.penguins.iter().find(|p| p.id == 1).unwrap();
    let p2 = engine.state.penguins.iter().find(|p| p.id == 2).unwrap();
    assert_eq!(p0.team, 0);
    assert_eq!(p1.team, 1);
    assert_eq!(p2.team, 1);

    // All three players must submit for the turn to be fully submitted
    engine.submit_action(0, Action::idle()).unwrap();
    engine.submit_action(1, Action::idle()).unwrap();
    let all_submitted = engine.submit_action(2, Action::idle()).unwrap();
    assert!(all_submitted, "all 3 submitted → should return true");

    // Full turn cycle completes without panic
    complete_one_idle_turn(&mut engine);
    assert!(is_planning(&engine.state.phase));
}

// ── Integration Test: run_until_phase_change helper works ────────────────────

#[test]
fn run_until_phase_change_helper_finds_resolving() {
    let mut engine = make_1v1();

    // Move to simulating (idle actions → will stop immediately)
    engine.tick(PLANNING_DEADLINE + 1.0);
    assert!(matches!(engine.state.phase, TurnPhase::Simulating));

    let evts = run_until_phase_change(&mut engine);
    assert!(
        evts.iter()
            .any(|e| matches!(e, GameEvent::PhaseChanged(TurnPhase::Resolving))),
        "helper should return the tick containing Resolving transition"
    );
}

// ── Integration Test: scores accumulate across multiple goals ─────────────────

#[test]
fn scores_accumulate_across_goals() {
    let goal_y = 200.0;

    // Two goals for team 0 in sequence
    for expected_score in [1u32, 2u32] {
        let ball_start = Vec2::new(790.0, goal_y);
        let players = vec![
            (0u8, 0u8, Vec2::new(200.0, 200.0)),
            (1u8, 1u8, Vec2::new(600.0, 200.0)),
        ];
        let mut engine = GameEngine::new(
            Field::new(800.0, 400.0),
            players,
            ball_start,
            120.0,
        );

        // Score `expected_score` goals by forcing ball into goal repeatedly
        let mut scored = 0u32;
        while scored < expected_score {
            engine.state.ball.pos = Vec2::new(790.0, goal_y);
            engine.state.ball.vel = Vec2::new(3000.0, 0.0);
            engine.state.phase = TurnPhase::Simulating;

            'inner: for _ in 0..2000 {
                let evts = engine.tick(DT);
                for e in &evts {
                    if matches!(e, GameEvent::GoalScored { .. }) {
                        scored += 1;
                        break 'inner;
                    }
                }
            }
        }

        assert_eq!(
            *engine.state.scores.get(&0u8).unwrap_or(&0),
            expected_score,
            "team 0 score should be {} after {} goals",
            expected_score,
            expected_score
        );
    }
}

// ── Integration Test: match ends with Draw when no goals ─────────────────────

#[test]
fn match_ends_draw_when_no_goals_scored() {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0u8, 0u8, Vec2::new(200.0, 200.0)),
        (1u8, 1u8, Vec2::new(600.0, 200.0)),
    ];
    let mut engine = GameEngine::new(field, players, Vec2::new(400.0, 200.0), 0.1);

    engine.state.phase = TurnPhase::Simulating;

    let mut result: Option<MatchResult> = None;
    for _ in 0..10_000 {
        let evts = engine.tick(DT);
        for e in &evts {
            if let GameEvent::MatchEnded(r) = e {
                result = Some(r.clone());
            }
        }
        if result.is_some() {
            break;
        }
        if matches!(engine.state.phase, TurnPhase::Planning { .. }) {
            engine.state.phase = TurnPhase::Simulating;
        }
    }

    assert_eq!(
        result.expect("MatchEnded should fire"),
        MatchResult::Draw,
        "no goals → Draw"
    );
}
