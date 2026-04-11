use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};

use penguin_soccer_engine::domain::types::PlayerId;

use crate::game_session::{GameCommand, GameSession};
use crate::lobby::RoomManager;
use crate::protocol::{ClientMessage, ServerMessage};

pub type AppState = Arc<Mutex<RoomManager>>;

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Channel for server->client messages
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn task to forward messages from channel to websocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Track this connection's state
    let mut current_room: Option<String> = None;
    let mut current_player_id: Option<PlayerId> = None;

    // Process incoming messages
    while let Some(Ok(msg)) = ws_stream.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                });
                continue;
            }
        };

        match client_msg {
            ClientMessage::CreateRoom => {
                let mut mgr = state.lock().await;
                let room_id = mgr.create_room();
                let player_id = mgr.join_room(&room_id, tx.clone()).unwrap();

                current_room = Some(room_id.clone());
                current_player_id = Some(player_id);

                let _ = tx.send(ServerMessage::RoomCreated {
                    room_id: room_id.clone(),
                });

                let room = mgr.rooms.get(&room_id).unwrap();
                let _ = tx.send(ServerMessage::RoomState {
                    players: room.player_infos(),
                    you: player_id,
                });
            }

            ClientMessage::JoinRoom { room_id } => {
                let mut mgr = state.lock().await;
                match mgr.join_room(&room_id, tx.clone()) {
                    Ok(player_id) => {
                        current_room = Some(room_id.clone());
                        current_player_id = Some(player_id);

                        let room = mgr.rooms.get(&room_id).unwrap();
                        let infos = room.player_infos();

                        // Send personalized RoomState to each player
                        for p in room.players.values() {
                            let _ = p.sender.send(ServerMessage::RoomState {
                                players: infos.clone(),
                                you: p.player_id,
                            });
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(ServerMessage::Error { message: e });
                    }
                }
            }

            ClientMessage::ChangeTeam { team } => {
                if let (Some(ref room_id), Some(player_id)) =
                    (&current_room, current_player_id)
                {
                    let mut mgr = state.lock().await;
                    if let Some(room) = mgr.rooms.get_mut(room_id) {
                        match room.change_team(player_id, team) {
                            Ok(()) => {
                                let infos = room.player_infos();
                                room.broadcast(&ServerMessage::RoomState {
                                    players: infos,
                                    you: player_id,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(ServerMessage::Error { message: e });
                            }
                        }
                    }
                }
            }

            ClientMessage::StartGame => {
                if let (Some(ref room_id), Some(_player_id)) =
                    (&current_room, current_player_id)
                {
                    let mut mgr = state.lock().await;
                    if let Some(room) = mgr.rooms.get_mut(room_id) {
                        if !room.can_start() {
                            let _ = tx.send(ServerMessage::Error {
                                message: "Cannot start: need at least 1 player per team"
                                    .to_string(),
                            });
                            continue;
                        }

                        room.game_active = true;

                        // Collect player data for game session
                        let players: Vec<_> = room
                            .players
                            .values()
                            .map(|p| (p.player_id, p.team, p.sender.clone()))
                            .collect();

                        let cmd_tx = GameSession::start(players);
                        room.game_cmd_tx = Some(cmd_tx);
                    }
                }
            }

            ClientMessage::SubmitAction { direction, power } => {
                if let (Some(ref room_id), Some(player_id)) =
                    (&current_room, current_player_id)
                {
                    let mgr = state.lock().await;
                    if let Some(room) = mgr.rooms.get(room_id) {
                        if let Some(ref cmd_tx) = room.game_cmd_tx {
                            let _ = cmd_tx.send(GameCommand::SubmitAction {
                                player_id,
                                direction,
                                power,
                            });
                        } else {
                            let _ = tx.send(ServerMessage::Error {
                                message: "No active game".to_string(),
                            });
                        }
                    }
                } else {
                    let _ = tx.send(ServerMessage::Error {
                        message: "No active game".to_string(),
                    });
                }
            }
        }
    }

    // Cleanup on disconnect
    if let (Some(room_id), Some(player_id)) = (current_room, current_player_id) {
        let mut mgr = state.lock().await;
        if let Some(room) = mgr.rooms.get_mut(&room_id) {
            room.remove_player(player_id);
            if room.players.is_empty() {
                mgr.rooms.remove(&room_id);
            }
        }
    }

    send_task.abort();
}
