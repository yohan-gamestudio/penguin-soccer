use std::collections::HashMap;

use penguin_soccer_engine::domain::types::{PlayerId, TeamId};
use serde::{Deserialize, Serialize};

// Client -> Server
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    CreateRoom,
    JoinRoom { room_id: String },
    ChangeTeam { team: TeamId },
    StartGame,
    SubmitAction { direction: [f64; 2], power: f64 },
}

// Server -> Client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    RoomCreated { room_id: String },
    RoomState { players: Vec<PlayerInfo>, you: PlayerId },
    Error { message: String },
    PhaseChanged { phase: String },
    PlanningStart { deadline: f64 },
    ActionConfirmed,
    SimulationFrame { penguins: Vec<EntityState>, ball: EntityState },
    GoalScored { scoring_team: TeamId, scores: HashMap<TeamId, u32> },
    MatchEnded { result: String, scores: HashMap<TeamId, u32> },
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub team: TeamId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityState {
    pub x: f64,
    pub y: f64,
}
