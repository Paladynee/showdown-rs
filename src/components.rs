use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

// TODO: add config system in serde_json and pull these values from there
// LazyCell, OnceCell get_or_init
pub const HTTP_PORT: u16 = 8080;
pub const WS_PORT: u16 = 8081;
pub const WS_TICKRATE: u32 = 120;
pub const PHYSICS_TICKRATE: u32 = 60;

pub fn get_public_directory() -> PathBuf {
    Path::new("./public/").canonicalize().unwrap()
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ServerMode {
    // hosts the server on 127.0.0.1
    Development,
    // hosts the server on local_ip_address::local_ip()
    Production,
    // asks from local_ip_address::enum
    Ask,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebSocketClientData {
    pub player: PartialPlayer,
    pub new_bullets: Vec<Bullet>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameState {
    pub players: HashMap<String, Player>,
    pub bullets: Vec<Bullet>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub velx: f32,
    pub vely: f32,
    pub life: f32,
    pub owner: String,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PartialPlayer {
    pub x: f32,
    pub y: f32,
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

impl Player {
    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.hp -= damage;
        if self.hp <= 0.0 {
            self.hp = 100.0;
            true
        } else {
            false
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
            bullets: vec![],
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

pub fn ask_network() -> IpAddr {
    loop {
        let networks = local_ip_address::list_afinet_netifas().unwrap();
        let question_string = format!(
            "Which network interface do you want to host the server on?\n\t{}\nAnswer (0-{}, anything else will reload): ",
            networks
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (i, network)| {
                    format!("{}{}: {} | {}\n\t", acc, i, network.0, network.1)
                }),
            networks.len() - 1
        );

        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut answer = String::new();

        stdout.write_all(question_string.as_bytes()).unwrap();
        stdout.flush().unwrap();

        answer.clear();
        stdin.read_line(&mut answer).unwrap();

        if let Ok(answer) = answer.trim().parse::<usize>() {
            if answer < networks.len() {
                return networks[answer].1;
            }
        }
    }
}
