use penguin_soccer_engine::domain::types::Vec2;
use penguin_soccer_engine::domain::penguin::Penguin;
use penguin_soccer_engine::domain::ball::Ball;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::physics::movement::{apply_movement_penguin, apply_movement_ball, all_stopped};
use penguin_soccer_engine::physics::collision::{
    collide_penguin_wall, collide_ball_wall,
    collide_penguin_penguin, collide_penguin_ball,
};

const DT: f64 = 1.0 / 60.0;

// === Step 1-1: 물리 이동 ===

#[test]
fn penguin_moves_when_velocity_set() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 100.0));
    penguin.vel = Vec2::new(100.0, 0.0);

    let old_x = penguin.pos.x;
    apply_movement_penguin(&mut penguin, DT);

    assert!(penguin.pos.x > old_x, "펭귄이 오른쪽으로 이동해야 함");
}

#[test]
fn penguin_decelerates_with_friction() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 100.0));
    penguin.vel = Vec2::new(200.0, 0.0);

    let initial_speed = penguin.vel.length();
    apply_movement_penguin(&mut penguin, DT);
    let after_speed = penguin.vel.length();

    assert!(after_speed < initial_speed, "마찰으로 속도가 줄어야 함");
}

#[test]
fn penguin_stops_below_threshold() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 100.0));
    penguin.vel = Vec2::new(0.5, 0.5); // Below threshold

    apply_movement_penguin(&mut penguin, DT);

    assert_eq!(penguin.vel.x, 0.0);
    assert_eq!(penguin.vel.y, 0.0);
}

#[test]
fn penguin_eventually_stops_after_many_ticks() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 100.0));
    penguin.vel = Vec2::new(300.0, 200.0);

    for _ in 0..1000 {
        apply_movement_penguin(&mut penguin, DT);
    }

    assert_eq!(penguin.vel.x, 0.0, "충분한 시간 후 정지해야 함");
    assert_eq!(penguin.vel.y, 0.0, "충분한 시간 후 정지해야 함");
}

#[test]
fn ball_moves_when_velocity_set() {
    let mut ball = Ball::new(Vec2::new(200.0, 200.0));
    ball.vel = Vec2::new(0.0, -150.0);

    let old_y = ball.pos.y;
    apply_movement_ball(&mut ball, DT);

    assert!(ball.pos.y < old_y, "공이 위로 이동해야 함");
}

#[test]
fn ball_decelerates_with_friction() {
    let mut ball = Ball::new(Vec2::new(200.0, 200.0));
    ball.vel = Vec2::new(300.0, 0.0);

    let initial_speed = ball.vel.length();
    apply_movement_ball(&mut ball, DT);

    assert!(ball.vel.length() < initial_speed, "공도 마찰으로 감속");
}

#[test]
fn all_stopped_returns_true_when_everything_stationary() {
    let penguins = vec![
        Penguin::new(0, 0, Vec2::new(100.0, 100.0)),
        Penguin::new(1, 1, Vec2::new(300.0, 100.0)),
    ];
    let ball = Ball::new(Vec2::new(200.0, 200.0));

    assert!(all_stopped(&penguins, &ball));
}

#[test]
fn all_stopped_returns_false_when_penguin_moving() {
    let mut penguins = vec![
        Penguin::new(0, 0, Vec2::new(100.0, 100.0)),
    ];
    penguins[0].vel = Vec2::new(50.0, 0.0);
    let ball = Ball::new(Vec2::new(200.0, 200.0));

    assert!(!all_stopped(&penguins, &ball));
}

#[test]
fn all_stopped_returns_false_when_ball_moving() {
    let penguins = vec![
        Penguin::new(0, 0, Vec2::new(100.0, 100.0)),
    ];
    let mut ball = Ball::new(Vec2::new(200.0, 200.0));
    ball.vel = Vec2::new(0.0, 30.0);

    assert!(!all_stopped(&penguins, &ball));
}

// === Step 1-2: 벽 충돌 ===

fn test_field() -> Field {
    Field::new(800.0, 400.0)
}

#[test]
fn penguin_bounces_off_right_wall() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(790.0, 200.0));
    penguin.vel = Vec2::new(100.0, 0.0);

    collide_penguin_wall(&mut penguin, &field);

    assert!(penguin.vel.x < 0.0, "오른쪽 벽에서 x 속도 반전");
    assert!(penguin.pos.x + penguin.radius <= field.width, "필드 안에 유지");
}

#[test]
fn penguin_bounces_off_left_wall() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(5.0, 200.0));
    penguin.vel = Vec2::new(-100.0, 0.0);

    collide_penguin_wall(&mut penguin, &field);

    assert!(penguin.vel.x > 0.0, "왼쪽 벽에서 x 속도 반전");
    assert!(penguin.pos.x - penguin.radius >= 0.0, "필드 안에 유지");
}

#[test]
fn penguin_bounces_off_top_wall() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(400.0, 5.0));
    penguin.vel = Vec2::new(0.0, -100.0);

    collide_penguin_wall(&mut penguin, &field);

    assert!(penguin.vel.y > 0.0, "상단 벽에서 y 속도 반전");
}

#[test]
fn penguin_bounces_off_bottom_wall() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(400.0, 395.0));
    penguin.vel = Vec2::new(0.0, 100.0);

    collide_penguin_wall(&mut penguin, &field);

    assert!(penguin.vel.y < 0.0, "하단 벽에서 y 속도 반전");
}

#[test]
fn ball_bounces_off_top_wall() {
    let field = test_field();
    let mut ball = Ball::new(Vec2::new(400.0, 3.0));
    ball.vel = Vec2::new(0.0, -200.0);

    collide_ball_wall(&mut ball, &field);

    assert!(ball.vel.y > 0.0, "공 상단 벽 반사");
}

#[test]
fn penguin_corner_collision_reverses_both_axes() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(795.0, 395.0));
    penguin.vel = Vec2::new(100.0, 100.0);

    collide_penguin_wall(&mut penguin, &field);

    assert!(penguin.vel.x < 0.0, "코너에서 x 반전");
    assert!(penguin.vel.y < 0.0, "코너에서 y 반전");
}

#[test]
fn penguin_stays_inside_field_after_many_bounces() {
    let field = test_field();
    let mut penguin = Penguin::new(0, 0, Vec2::new(400.0, 200.0));
    penguin.vel = Vec2::new(500.0, 300.0);

    for _ in 0..500 {
        apply_movement_penguin(&mut penguin, DT);
        collide_penguin_wall(&mut penguin, &field);
    }

    assert!(penguin.pos.x - penguin.radius >= 0.0);
    assert!(penguin.pos.x + penguin.radius <= field.width);
    assert!(penguin.pos.y - penguin.radius >= 0.0);
    assert!(penguin.pos.y + penguin.radius <= field.height);
}

// === Step 1-3: 원-원 충돌 ===

#[test]
fn penguin_hits_ball_head_on_transfers_momentum() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 200.0));
    penguin.vel = Vec2::new(200.0, 0.0);
    let mut ball = Ball::new(Vec2::new(124.0, 200.0)); // distance=24 < 25, clearly overlapping

    collide_penguin_ball(&mut penguin, &mut ball);

    assert!(ball.vel.x > 0.0, "공이 오른쪽으로 밀려야 함");
    assert!(penguin.vel.x < 200.0, "펭귄 속도가 줄어야 함");
}

#[test]
fn penguin_penguin_collision_both_bounce() {
    let mut a = Penguin::new(0, 0, Vec2::new(100.0, 200.0));
    a.vel = Vec2::new(200.0, 0.0);
    let mut b = Penguin::new(1, 1, Vec2::new(129.0, 200.0)); // touching (15+15=30)
    b.vel = Vec2::ZERO;

    collide_penguin_penguin(&mut a, &mut b);

    assert!(b.vel.x > 0.0, "맞은 펭귄이 밀려남");
    assert!(a.vel.x < 200.0, "친 펭귄 속도 감소");
}

#[test]
fn oblique_collision_splits_direction() {
    let mut penguin = Penguin::new(0, 0, Vec2::new(100.0, 200.0));
    penguin.vel = Vec2::new(200.0, 0.0);
    // Ball slightly above — oblique hit
    let mut ball = Ball::new(Vec2::new(118.0, 188.0)); // distance=sqrt(324+144)≈21.4 < 25, clearly overlapping

    collide_penguin_ball(&mut penguin, &mut ball);

    // Ball should have both x and y velocity components from oblique collision
    assert!(ball.vel.x != 0.0 && ball.vel.y != 0.0, "비스듬한 충돌로 방향 분리");
}

#[test]
fn overlapping_circles_get_separated() {
    let mut a = Penguin::new(0, 0, Vec2::new(100.0, 200.0));
    let mut b = Penguin::new(1, 1, Vec2::new(110.0, 200.0)); // overlapping (dist=10, need 30)

    collide_penguin_penguin(&mut a, &mut b);

    let dist = (b.pos - a.pos).length();
    let min_dist = a.radius + b.radius;
    assert!(
        dist >= min_dist - 0.01,
        "충돌 후 분리되어야 함: dist={}, min={}",
        dist,
        min_dist
    );
}

#[test]
fn no_collision_when_circles_far_apart() {
    let mut a = Penguin::new(0, 0, Vec2::new(100.0, 200.0));
    a.vel = Vec2::new(50.0, 0.0);
    let mut b = Penguin::new(1, 1, Vec2::new(200.0, 200.0));
    b.vel = Vec2::ZERO;

    let vel_b_before = b.vel;
    collide_penguin_penguin(&mut a, &mut b);

    assert_eq!(b.vel.x, vel_b_before.x, "멀리 떨어진 펭귄은 영향 없음");
}
