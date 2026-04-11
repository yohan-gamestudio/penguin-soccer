use std::collections::HashMap;

use tokio::sync::mpsc;
use uuid::Uuid;

use penguin_soccer_engine::domain::types::PlayerId;

use crate::protocol::ServerMessage;
use crate::room::Room;

pub struct RoomManager {
    pub rooms: HashMap<String, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self) -> String {
        let room_id = Uuid::new_v4().to_string()[..8].to_string();
        let room = Room::new(room_id.clone());
        self.rooms.insert(room_id.clone(), room);
        room_id
    }

    pub fn join_room(
        &mut self,
        room_id: &str,
        sender: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<PlayerId, String> {
        let room = self
            .rooms
            .get_mut(room_id)
            .ok_or_else(|| "Room not found".to_string())?;
        room.add_player(sender)
    }

    pub fn list_rooms(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }
}
