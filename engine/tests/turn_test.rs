use penguin_soccer_engine::domain::action::Action;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::GameEvent;

const DT: f64 = 1.0 / 60.0;

fn make_1v1_engine() -> GameEngine {
    let field = Field::new(800.0, 400.0);
    let players = vec![
        (0, 0, Vec2::new(200.0, 200.0)),
        (1, 1, Vec2::new(600.0, 200.0)),
    ];
    let ball_pos = Vec2::new(400.0, 200.0);
    GameEngine::new(field, players, ball_pos, 120.0)
}

fn is_planning(phase: &TurnPhase) -> bool {
    matches!(phase, TurnPhase::Planning { .. })
}

fn is_simulating(phase: &TurnPhase) -> bool {
    matches!(phase, TurnPhase::Simulating)
}

fn is_resolving(phase: &TurnPhase) -> bool {
    matches!(phase, TurnPhase::Resolving)
}

// === Step 1-4: 턴 시스템 ===

#[test]
fn initial_phase_is_planning() {
    let engine = make_1v1_engine();
    assert!(
        is_planning(&engine.state.phase),
        "초기 페이즈는 Planning이어야 함"
    );
}

#[test]
fn all_players_submit_returns_true() {
    let mut engine = make_1v1_engine();

    let action = Action::new(Vec2::new(1.0, 0.0), 0.5);

    let first = engine.submit_action(0, action).expect("첫 제출 성공");
    assert!(!first, "1명만 제출했으면 false");

    let second = engine.submit_action(1, action).expect("두 번째 제출 성공");
    assert!(second, "모두 제출했으면 true");
}

#[test]
fn all_players_submit_transitions_to_simulating_on_tick() {
    let mut engine = make_1v1_engine();

    let action = Action::new(Vec2::new(1.0, 0.0), 0.5);
    engine.submit_action(0, action).unwrap();
    engine.submit_action(1, action).unwrap();

    // tick이 시뮬레이션을 시작하려면 planning이 timed-out이거나
    // 모든 플레이어가 제출 후 tick에서 전환이 발생해야 함.
    // 엔진 로직: Planning tick은 timeout만 확인함 → 모두 제출 후 submit_action이
    // all_submitted=true를 반환하지만, 실제 phase 전환은 tick 외부에서 해야 하는지
    // 아니면 timeout 경로를 통해 일어나는지 확인.
    //
    // engine.rs 분석: tick의 Planning 분기는 check_timeout만 호출.
    // submit_action은 all_submitted bool만 반환하고 phase를 바꾸지 않음.
    // 따라서 모두 제출 후 timeout을 강제로 발생시켜야 Simulating으로 전환됨.
    // → deadline을 초과하는 dt로 tick.
    let events = engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);

    let has_simulating_event = events.iter().any(|e| {
        matches!(e, GameEvent::PhaseChanged(TurnPhase::Simulating))
    });
    assert!(
        has_simulating_event,
        "timeout 후 Simulating PhaseChanged 이벤트 발생해야 함"
    );
    assert!(
        is_simulating(&engine.state.phase),
        "tick 후 페이즈가 Simulating이어야 함"
    );
}

#[test]
fn partial_submit_then_timeout_fills_idle_and_transitions() {
    let mut engine = make_1v1_engine();

    // 플레이어 0만 제출
    let action = Action::new(Vec2::new(0.0, 1.0), 1.0);
    engine.submit_action(0, action).unwrap();

    // deadline 초과 tick → 플레이어 1은 idle 채워짐
    let events = engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);

    let has_simulating_event = events.iter().any(|e| {
        matches!(e, GameEvent::PhaseChanged(TurnPhase::Simulating))
    });
    assert!(
        has_simulating_event,
        "부분 제출 + timeout → Simulating 전환 이벤트"
    );
    assert!(
        is_simulating(&engine.state.phase),
        "부분 제출 + timeout 후 페이즈가 Simulating이어야 함"
    );
}

#[test]
fn duplicate_submit_returns_error() {
    let mut engine = make_1v1_engine();

    let action = Action::new(Vec2::new(1.0, 0.0), 0.5);
    engine.submit_action(0, action).unwrap();

    let result = engine.submit_action(0, action);
    assert!(
        result.is_err(),
        "같은 플레이어가 두 번 제출하면 에러여야 함"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("AlreadySubmitted"),
        "에러 메시지에 AlreadySubmitted 포함: {}",
        err
    );
}

#[test]
fn submit_during_simulating_returns_error() {
    let mut engine = make_1v1_engine();

    // timeout으로 Simulating 진입
    engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);
    assert!(is_simulating(&engine.state.phase));

    let action = Action::new(Vec2::new(1.0, 0.0), 0.5);
    let result = engine.submit_action(0, action);
    assert!(
        result.is_err(),
        "Simulating 중 제출은 에러여야 함"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("NotInPlanningPhase"),
        "에러 메시지에 NotInPlanningPhase 포함: {}",
        err
    );
}

#[test]
fn simulating_to_resolving_when_all_stopped() {
    let mut engine = make_1v1_engine();

    // timeout → Simulating (idle actions → 아무도 움직이지 않음)
    engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);
    assert!(is_simulating(&engine.state.phase));

    // 모든 객체가 이미 정지 상태(idle action) → 다음 tick에서 Resolving 전환
    let events = engine.tick(DT);

    let has_resolving_event = events.iter().any(|e| {
        matches!(e, GameEvent::PhaseChanged(TurnPhase::Resolving))
    });
    assert!(
        has_resolving_event,
        "모든 객체 정지 시 Resolving PhaseChanged 이벤트 발생해야 함"
    );
    assert!(
        is_resolving(&engine.state.phase),
        "모든 객체 정지 후 페이즈가 Resolving이어야 함"
    );
}

#[test]
fn resolving_transitions_to_planning_on_tick() {
    let mut engine = make_1v1_engine();

    // Planning → Simulating
    engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);
    // Simulating → Resolving
    engine.tick(DT);
    assert!(is_resolving(&engine.state.phase));

    // Resolving → Planning
    let events = engine.tick(DT);

    let has_planning_event = events.iter().any(|e| {
        matches!(e, GameEvent::PhaseChanged(TurnPhase::Planning { .. }))
    });
    assert!(
        has_planning_event,
        "Resolving tick 후 Planning PhaseChanged 이벤트 발생해야 함"
    );
    assert!(
        is_planning(&engine.state.phase),
        "Resolving 후 페이즈가 다시 Planning이어야 함"
    );
}

#[test]
fn full_turn_cycle_completes_planning_simulating_resolving_planning() {
    let mut engine = make_1v1_engine();

    // 1) Planning
    assert!(is_planning(&engine.state.phase));

    // 2) timeout → Simulating
    engine.tick(penguin_soccer_engine::rules::turn::PLANNING_DEADLINE + 1.0);
    assert!(is_simulating(&engine.state.phase), "Simulating 단계");

    // 3) all stopped → Resolving
    engine.tick(DT);
    assert!(is_resolving(&engine.state.phase), "Resolving 단계");

    // 4) Resolving → Planning
    engine.tick(DT);
    assert!(is_planning(&engine.state.phase), "다음 Planning 단계");
}
