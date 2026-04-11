use std::collections::HashMap;
use std::time::Duration;

use penguin_soccer_engine::domain::action::Action;
use penguin_soccer_engine::domain::field::Field;
use penguin_soccer_engine::domain::game_state::TurnPhase;
use penguin_soccer_engine::domain::types::{PlayerId, TeamId, Vec2};
use penguin_soccer_engine::engine::GameEngine;
use penguin_soccer_engine::ports::event::GameEvent;
use penguin_soccer_engine::rules::match_flow::MatchResult;
use tokio::sync::mpsc;

use crate::protocol::{EntityState, ServerMessage};

const FIELD_WIDTH: f64 = 800.0;
const FIELD_HEIGHT: f64 = 400.0;
const BALL_CENTER_X: f64 = 400.0;
const BALL_CENTER_Y: f64 = 200.0;
const MATCH_DURATION: f64 = 120.0;
const PLANNING_DEADLINE: f64 = 30.0;
const SIM_DT: f64 = 1.0 / 60.0;

#[derive(Debug)]
pub enum GameCommand {
    SubmitAction {
        player_id: PlayerId,
        direction: [f64; 2],
        power: f64,
    },
}

pub struct GameSession {
    engine: GameEngine,
    player_senders: HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>,
    command_rx: mpsc::UnboundedReceiver<GameCommand>,
}

impl GameSession {
    pub fn start(
        players: Vec<(PlayerId, TeamId, mpsc::UnboundedSender<ServerMessage>)>,
    ) -> mpsc::UnboundedSender<GameCommand> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let field = Field::new(FIELD_WIDTH, FIELD_HEIGHT);
        let ball_pos = Vec2::new(BALL_CENTER_X, BALL_CENTER_Y);

        // Assign starting positions
        let mut engine_players = Vec::new();
        let mut player_senders = HashMap::new();

        let mut team0_idx = 0u8;
        let mut team1_idx = 0u8;

        for (pid, team, sender) in &players {
            let pos = if *team == 0 {
                let p = team0_starting_pos(team0_idx);
                team0_idx += 1;
                p
            } else {
                let p = team1_starting_pos(team1_idx);
                team1_idx += 1;
                p
            };
            engine_players.push((*pid, *team, pos));
            player_senders.insert(*pid, sender.clone());
        }

        let engine = GameEngine::new(field, engine_players, ball_pos, MATCH_DURATION);

        let session = GameSession {
            engine,
            player_senders,
            command_rx: cmd_rx,
        };

        tokio::spawn(session.run());

        cmd_tx
    }

    async fn run(mut self) {
        // Broadcast initial planning phase
        self.broadcast(&ServerMessage::PlanningStart {
            deadline: PLANNING_DEADLINE,
        });
        self.broadcast(&ServerMessage::PhaseChanged {
            phase: "planning".to_string(),
        });

        let mut interval = tokio::time::interval(Duration::from_secs_f64(SIM_DT));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let events = self.engine.tick(SIM_DT);

                    // During simulation, broadcast frame state
                    if matches!(self.engine.state.phase, TurnPhase::Simulating) {
                        self.broadcast_frame();
                    }

                    for event in events {
                        self.handle_event(&event);
                    }
                }
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(GameCommand::SubmitAction { player_id, direction, power }) => {
                            let dir = Vec2::new(direction[0], direction[1]);
                            let action = Action::new(dir, power);
                            match self.engine.submit_action(player_id, action) {
                                Ok(all_submitted) => {
                                    self.send_to(player_id, ServerMessage::ActionConfirmed);
                                    if all_submitted {
                                        // All submitted; the next tick() will transition to Simulating
                                    }
                                }
                                Err(e) => {
                                    self.send_to(player_id, ServerMessage::Error { message: e });
                                }
                            }
                        }
                        None => {
                            // All command senders dropped, end session
                            break;
                        }
                    }
                }
            }
        }
    }

    fn handle_event(&self, event: &GameEvent) {
        match event {
            GameEvent::PhaseChanged(phase) => {
                let phase_str = match phase {
                    TurnPhase::Planning { .. } => "planning",
                    TurnPhase::Simulating => "simulating",
                    TurnPhase::Resolving => "resolving",
                };
                self.broadcast(&ServerMessage::PhaseChanged {
                    phase: phase_str.to_string(),
                });
                if matches!(phase, TurnPhase::Planning { .. }) {
                    self.broadcast(&ServerMessage::PlanningStart {
                        deadline: PLANNING_DEADLINE,
                    });
                }
            }
            GameEvent::GoalScored { scoring_team } => {
                self.broadcast(&ServerMessage::GoalScored {
                    scoring_team: *scoring_team,
                    scores: self.engine.state.scores.clone(),
                });
            }
            GameEvent::MatchEnded(result) => {
                let result_str = match result {
                    MatchResult::Win(team) => format!("team_{}_wins", team),
                    MatchResult::Draw => "draw".to_string(),
                };
                self.broadcast(&ServerMessage::MatchEnded {
                    result: result_str,
                    scores: self.engine.state.scores.clone(),
                });
            }
        }
    }

    fn broadcast_frame(&self) {
        let penguins: Vec<EntityState> = self
            .engine
            .state
            .penguins
            .iter()
            .map(|p| EntityState {
                x: p.pos.x,
                y: p.pos.y,
            })
            .collect();

        let ball = EntityState {
            x: self.engine.state.ball.pos.x,
            y: self.engine.state.ball.pos.y,
        };

        self.broadcast(&ServerMessage::SimulationFrame { penguins, ball });
    }

    fn broadcast(&self, msg: &ServerMessage) {
        for sender in self.player_senders.values() {
            let _ = sender.send(msg.clone());
        }
    }

    fn send_to(&self, player_id: PlayerId, msg: ServerMessage) {
        if let Some(sender) = self.player_senders.get(&player_id) {
            let _ = sender.send(msg);
        }
    }
}

fn team0_starting_pos(idx: u8) -> Vec2 {
    match idx {
        0 => Vec2::new(200.0, 150.0),
        1 => Vec2::new(200.0, 250.0),
        _ => Vec2::new(150.0, 200.0),
    }
}

fn team1_starting_pos(idx: u8) -> Vec2 {
    match idx {
        0 => Vec2::new(600.0, 150.0),
        1 => Vec2::new(600.0, 250.0),
        _ => Vec2::new(650.0, 200.0),
    }
}
