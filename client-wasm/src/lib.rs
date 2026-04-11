use penguin_soccer_engine::domain::action::{Action, MAX_POWER};
use penguin_soccer_engine::domain::types::Vec2;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_DRAG_DISTANCE: f64 = 150.0;
const FRICTION: f64 = 0.98;
const STOP_THRESHOLD: f64 = 1.0;
const PENGUIN_RADIUS: f64 = 15.0;

// ── Serialisable output types ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ActionResult {
    pub direction: [f64; 2],
    pub power: f64,
}

#[derive(Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// ── calculate_action ─────────────────────────────────────────────────────────

/// Convert a drag gesture into an Action and return { direction, power }.
///
/// Slingshot convention: the penguin shoots in the direction opposite to the
/// drag, so direction = normalise(start − end).
#[wasm_bindgen]
pub fn calculate_action(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
) -> JsValue {
    let drag = Vec2::new(start_x - end_x, start_y - end_y);
    let distance = drag.length();
    let power = (distance / MAX_DRAG_DISTANCE).clamp(0.0, 1.0);
    let direction = drag.normalized();

    // Build a validated Action through the engine (normalises direction,
    // clamps power) – then pull the fields back out for the JS result.
    let action = Action::new(direction, power);

    let result = ActionResult {
        direction: [action.direction.x, action.direction.y],
        power: action.power,
    };

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// ── predict_trajectory ───────────────────────────────────────────────────────

/// Run a lightweight single-penguin simulation and return an array of {x, y}
/// points for the first `max_frames` steps (default 60).
///
/// The simulation mirrors the engine physics:
///   - pos += vel * dt  (dt = 1/60 s)
///   - vel *= FRICTION
///   - wall bounce (elastic, clamped to field bounds)
///   - stops early if speed falls below STOP_THRESHOLD
#[wasm_bindgen]
pub fn predict_trajectory(
    x: f64,
    y: f64,
    dir_x: f64,
    dir_y: f64,
    power: f64,
    field_w: f64,
    field_h: f64,
) -> JsValue {
    const DT: f64 = 1.0 / 60.0;
    const MAX_FRAMES: usize = 60;

    // Build initial velocity the same way the engine does.
    let direction = Vec2::new(dir_x, dir_y).normalized();
    let action = Action::new(direction, power.clamp(0.0, 1.0));
    let mut vel = action.to_velocity();
    let mut pos = Vec2::new(x, y);

    let mut points: Vec<Point> = Vec::with_capacity(MAX_FRAMES + 1);

    // Record the starting position.
    points.push(Point { x: pos.x, y: pos.y });

    for _ in 0..MAX_FRAMES {
        if vel.length_squared() < STOP_THRESHOLD * STOP_THRESHOLD {
            break;
        }

        // Move
        pos = pos + vel * DT;
        vel = vel * FRICTION;

        // Wall bounce (same logic as collision::collide_circle_wall)
        if pos.x - PENGUIN_RADIUS < 0.0 {
            pos.x = PENGUIN_RADIUS;
            vel.x = vel.x.abs();
        }
        if pos.x + PENGUIN_RADIUS > field_w {
            pos.x = field_w - PENGUIN_RADIUS;
            vel.x = -vel.x.abs();
        }
        if pos.y - PENGUIN_RADIUS < 0.0 {
            pos.y = PENGUIN_RADIUS;
            vel.y = vel.y.abs();
        }
        if pos.y + PENGUIN_RADIUS > field_h {
            pos.y = field_h - PENGUIN_RADIUS;
            vel.y = -vel.y.abs();
        }

        points.push(Point { x: pos.x, y: pos.y });
    }

    serde_wasm_bindgen::to_value(&points).unwrap_or(JsValue::NULL)
}

// ── suppress unused-import warning for MAX_POWER ─────────────────────────────
// MAX_POWER is re-exported so JS can read the engine constant if needed.
#[wasm_bindgen]
pub fn max_power() -> f64 {
    MAX_POWER
}
