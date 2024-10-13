use std::{env, thread, sync::{Arc, Mutex}};

mod firsttry;
mod http_server;
mod physics_server;
mod ws_server;

use http_server::create_http_server;
use physics_server::{create_physics_server, GameState};
use ws_server::create_ws_server;

pub const HTTP_PORT: u16 = 8080;
pub const WS_PORT: u16 = 8081;
pub const WS_TICKRATE: u32 = 30;
pub const PHYSICS_TICKRATE: u32 = 60;

pub const PUBLIC_DIRECTORY: &str = "./public/";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ServerMode {
    // hosts the server on 127.0.0.1
    Development,
    // hosts the server on local_ip_address::local_ip()
    Production,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let server_mode = if args.len() == 2 && args[1] == "prod" {
        ServerMode::Production
    } else {
        ServerMode::Development
    };

    let master_game_state = Arc::new(Mutex::new(GameState::new()));

    let mut threads = vec![];

    let http_server = thread::spawn(move || {
        create_http_server(server_mode);
    });

    let ws_game_state_clone = master_game_state.clone();
    let ws_server = thread::spawn(move || {
        create_ws_server(server_mode, ws_game_state_clone);
    });

    let physics_game_state_clone = master_game_state.clone();
    let physics_server = thread::spawn(move || {
        create_physics_server(physics_game_state_clone);
    });

    threads.push(http_server);
    threads.push(ws_server);
    threads.push(physics_server);

    for thread in threads {
        thread.join().unwrap();
    }
}
