use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ServerMode {
    // hosts the server on 127.0.0.1
    Development,
    // hosts the server on local_ip_address::local_ip()
    Production,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketClientData {
    pub player: Player,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameState {
    pub players: HashMap<String, Player>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameStatePacket<'a> {
    pub recipient: &'a str,
    pub game_state: GameState,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
}

impl WebSocketClientData {
    pub fn is_valid(&self) -> bool {
        if self.player.hp < 0.0 || self.player.hp > 100.0 {
            return false;
        }

        true
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            hp: 100.0,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
        }
    }
}

impl<'a> GameStatePacket<'a> {
    pub fn new(game_state: GameState, ws_identifier: &'a str) -> Self {
        GameStatePacket {
            recipient: ws_identifier,
            game_state,
        }
    }
}
