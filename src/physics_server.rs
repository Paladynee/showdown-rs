use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::PHYSICS_TICKRATE;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameState {
    pub players: HashMap<String, Player>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameStatePacket<'a> {
    pub recipient: &'a str,
    pub game_state: GameState,
}

impl<'a> GameStatePacket<'a> {
    pub fn new(game_state: GameState, ws_identifier: &'a str) -> Self {
        GameStatePacket {
            recipient: ws_identifier,
            game_state,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
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

pub fn create_physics_server(game_state: Arc<Mutex<GameState>>) {
    let last_update = Instant::now();
    let update_interval = Duration::from_secs(1) / PHYSICS_TICKRATE;

    loop {
        update_physics(&game_state);

        let now = Instant::now();
        if now - last_update < update_interval {
            let sleep_duration = update_interval - (now - last_update);
            std::thread::sleep(sleep_duration);
        }
    }
}

fn update_physics(_game_state: &Arc<Mutex<GameState>>) {
    // for now, no physics. in the future, bullets will go according to their velocity etc.
}
