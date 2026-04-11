use std::collections::HashMap;

use penguin_soccer_engine::domain::types::{PlayerId, TeamId};
use tokio::sync::mpsc;

use crate::game_session::GameCommand;
use crate::protocol::{PlayerInfo, ServerMessage};

pub struct PlayerConnection {
    pub player_id: PlayerId,
    pub team: TeamId,
    pub name: String,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

pub struct Room {
    pub id: String,
    pub players: HashMap<PlayerId, PlayerConnection>,
    pub game_active: bool,
    pub game_cmd_tx: Option<mpsc::UnboundedSender<GameCommand>>,
    next_player_id: PlayerId,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: HashMap::new(),
            game_active: false,
            game_cmd_tx: None,
            next_player_id: 0,
        }
    }

    pub fn add_player(
        &mut self,
        sender: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<PlayerId, String> {
        if self.is_full() {
            return Err("Room is full (max 4 players)".to_string());
        }
        if self.game_active {
            return Err("Game already in progress".to_string());
        }

        let player_id = self.next_player_id;
        self.next_player_id += 1;

        // Assign team: alternate between 0 and 1
        let team: TeamId = if self.players.values().filter(|p| p.team == 0).count()
            <= self.players.values().filter(|p| p.team == 1).count()
        {
            0
        } else {
            1
        };

        let name = format!("Penguin {}", player_id + 1);

        self.players.insert(
            player_id,
            PlayerConnection {
                player_id,
                team,
                name,
                sender,
            },
        );

        Ok(player_id)
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn change_team(&mut self, player_id: PlayerId, team: TeamId) -> Result<(), String> {
        if self.game_active {
            return Err("Cannot change team during game".to_string());
        }
        if let Some(player) = self.players.get_mut(&player_id) {
            player.team = team;
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= 4
    }

    pub fn can_start(&self) -> bool {
        // Need at least 1 player on each team
        let team0 = self.players.values().filter(|p| p.team == 0).count();
        let team1 = self.players.values().filter(|p| p.team == 1).count();
        team0 >= 1 && team1 >= 1 && !self.game_active
    }

    pub fn player_infos(&self) -> Vec<PlayerInfo> {
        self.players
            .values()
            .map(|p| PlayerInfo {
                id: p.player_id,
                team: p.team,
                name: p.name.clone(),
            })
            .collect()
    }

    pub fn broadcast(&self, msg: &ServerMessage) {
        for player in self.players.values() {
            let _ = player.sender.send(msg.clone());
        }
    }

    pub fn send_to(&self, player_id: PlayerId, msg: ServerMessage) {
        if let Some(player) = self.players.get(&player_id) {
            let _ = player.sender.send(msg);
        }
    }
}
