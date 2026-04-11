use std::collections::HashMap;

use penguin_soccer_engine::domain::types::{PlayerId, TeamId};
use tokio::sync::mpsc;

use crate::game_session::GameCommand;
use crate::protocol::{PlayerInfo, ServerMessage};

// Pure domain state (testable, no async/channel deps)
#[derive(Debug, Clone, PartialEq)]
pub enum RoomPhase {
    Lobby,
    Playing,
    Ended,
}

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub player_id: PlayerId,
    pub team: TeamId,
    pub name: String,
    pub ready: bool,
}

// Infrastructure wrapper
pub struct PlayerConnection {
    pub data: PlayerData,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

pub struct Room {
    pub id: String,
    pub players: HashMap<PlayerId, PlayerConnection>,
    pub state: RoomPhase,
    pub game_cmd_tx: Option<mpsc::UnboundedSender<GameCommand>>,
    next_player_id: PlayerId,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: HashMap::new(),
            state: RoomPhase::Lobby,
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
        if self.state != RoomPhase::Lobby {
            return Err("Game already in progress".to_string());
        }

        let player_id = self.next_player_id;
        self.next_player_id += 1;

        // Assign team: balance teams
        let team: TeamId = if self.players.values().filter(|p| p.data.team == 0).count()
            <= self.players.values().filter(|p| p.data.team == 1).count()
        {
            0
        } else {
            1
        };

        let name = format!("Penguin {}", player_id + 1);

        self.players.insert(
            player_id,
            PlayerConnection {
                data: PlayerData {
                    player_id,
                    team,
                    name,
                    ready: false,
                },
                sender,
            },
        );

        Ok(player_id)
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn change_team(&mut self, player_id: PlayerId, team: TeamId) -> Result<(), String> {
        if self.state != RoomPhase::Lobby {
            return Err("Cannot change team during game".to_string());
        }
        if let Some(player) = self.players.get_mut(&player_id) {
            player.data.team = team;
            player.data.ready = false; // Reset ready on team change
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }

    pub fn toggle_ready(&mut self, player_id: PlayerId) -> Result<bool, String> {
        if self.state != RoomPhase::Lobby {
            return Err("Cannot toggle ready during game".to_string());
        }
        if let Some(player) = self.players.get_mut(&player_id) {
            player.data.ready = !player.data.ready;
            Ok(player.data.ready)
        } else {
            Err("Player not found".to_string())
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= 4
    }

    /// Check if game can start: all players ready + at least 1 per team
    pub fn can_start(&self) -> bool {
        if self.state != RoomPhase::Lobby || self.players.len() < 2 {
            return false;
        }
        let all_ready = self.players.values().all(|p| p.data.ready);
        let team0 = self.players.values().filter(|p| p.data.team == 0).count();
        let team1 = self.players.values().filter(|p| p.data.team == 1).count();
        all_ready && team0 >= 1 && team1 >= 1
    }

    pub fn start_game(&mut self) {
        self.state = RoomPhase::Playing;
    }

    pub fn end_game(&mut self) {
        self.state = RoomPhase::Ended;
        self.game_cmd_tx = None;
    }

    pub fn return_to_lobby(&mut self) {
        self.state = RoomPhase::Lobby;
        self.game_cmd_tx = None;
        // Reset all players ready state
        for player in self.players.values_mut() {
            player.data.ready = false;
        }
    }

    pub fn player_infos(&self) -> Vec<PlayerInfo> {
        self.players
            .values()
            .map(|p| PlayerInfo {
                id: p.data.player_id,
                team: p.data.team,
                name: p.data.name.clone(),
                ready: p.data.ready,
            })
            .collect()
    }

    pub fn state_str(&self) -> &'static str {
        match self.state {
            RoomPhase::Lobby => "lobby",
            RoomPhase::Playing => "playing",
            RoomPhase::Ended => "ended",
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sender() -> mpsc::UnboundedSender<ServerMessage> {
        let (tx, _rx) = mpsc::unbounded_channel();
        tx
    }

    #[test]
    fn test_new_room_is_in_lobby_state() {
        let room = Room::new("test".to_string());
        assert_eq!(room.state, RoomPhase::Lobby);
        assert!(room.players.is_empty());
    }

    #[test]
    fn test_add_player_assigns_balanced_teams() {
        let mut room = Room::new("test".to_string());
        let p0 = room.add_player(make_sender()).unwrap();
        let p1 = room.add_player(make_sender()).unwrap();

        assert_eq!(room.players.get(&p0).unwrap().data.team, 0);
        assert_eq!(room.players.get(&p1).unwrap().data.team, 1);
    }

    #[test]
    fn test_cannot_join_full_room() {
        let mut room = Room::new("test".to_string());
        for _ in 0..4 {
            room.add_player(make_sender()).unwrap();
        }
        assert!(room.add_player(make_sender()).is_err());
    }

    #[test]
    fn test_cannot_join_during_game() {
        let mut room = Room::new("test".to_string());
        room.add_player(make_sender()).unwrap();
        room.start_game();
        assert!(room.add_player(make_sender()).is_err());
    }

    #[test]
    fn test_toggle_ready() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();

        assert!(!room.players.get(&pid).unwrap().data.ready);
        room.toggle_ready(pid).unwrap();
        assert!(room.players.get(&pid).unwrap().data.ready);
        room.toggle_ready(pid).unwrap();
        assert!(!room.players.get(&pid).unwrap().data.ready);
    }

    #[test]
    fn test_cannot_toggle_ready_during_game() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();
        room.start_game();
        assert!(room.toggle_ready(pid).is_err());
    }

    #[test]
    fn test_can_start_requires_all_ready_and_both_teams() {
        let mut room = Room::new("test".to_string());
        let p0 = room.add_player(make_sender()).unwrap();
        let p1 = room.add_player(make_sender()).unwrap();

        // Not ready yet
        assert!(!room.can_start());

        // Only one ready
        room.toggle_ready(p0).unwrap();
        assert!(!room.can_start());

        // Both ready
        room.toggle_ready(p1).unwrap();
        assert!(room.can_start());
    }

    #[test]
    fn test_can_start_requires_both_teams() {
        let mut room = Room::new("test".to_string());
        let p0 = room.add_player(make_sender()).unwrap();
        let p1 = room.add_player(make_sender()).unwrap();

        // Put both on same team
        room.change_team(p1, 0).unwrap();
        room.toggle_ready(p0).unwrap();
        room.toggle_ready(p1).unwrap();

        assert!(!room.can_start()); // Both on team 0, no one on team 1
    }

    #[test]
    fn test_change_team_resets_ready() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();
        room.toggle_ready(pid).unwrap();
        assert!(room.players.get(&pid).unwrap().data.ready);

        room.change_team(pid, 1).unwrap();
        assert!(!room.players.get(&pid).unwrap().data.ready);
    }

    #[test]
    fn test_state_transitions() {
        let mut room = Room::new("test".to_string());
        assert_eq!(room.state, RoomPhase::Lobby);

        room.start_game();
        assert_eq!(room.state, RoomPhase::Playing);

        room.end_game();
        assert_eq!(room.state, RoomPhase::Ended);

        room.return_to_lobby();
        assert_eq!(room.state, RoomPhase::Lobby);
        // All players should be not-ready after return
        for p in room.players.values() {
            assert!(!p.data.ready);
        }
    }

    #[test]
    fn test_return_to_lobby_resets_ready() {
        let mut room = Room::new("test".to_string());
        let p0 = room.add_player(make_sender()).unwrap();
        let p1 = room.add_player(make_sender()).unwrap();
        room.toggle_ready(p0).unwrap();
        room.toggle_ready(p1).unwrap();
        room.start_game();
        room.end_game();
        room.return_to_lobby();

        assert!(!room.players.get(&p0).unwrap().data.ready);
        assert!(!room.players.get(&p1).unwrap().data.ready);
        assert_eq!(room.state, RoomPhase::Lobby);
    }

    #[test]
    fn test_player_infos_includes_ready() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();

        let infos = room.player_infos();
        assert!(!infos[0].ready);

        room.toggle_ready(pid).unwrap();
        let infos = room.player_infos();
        assert!(infos[0].ready);
    }

    #[test]
    fn test_remove_player() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();
        assert_eq!(room.players.len(), 1);
        room.remove_player(pid);
        assert_eq!(room.players.len(), 0);
    }

    #[test]
    fn test_cannot_change_team_during_game() {
        let mut room = Room::new("test".to_string());
        let pid = room.add_player(make_sender()).unwrap();
        room.start_game();
        assert!(room.change_team(pid, 1).is_err());
    }
}
