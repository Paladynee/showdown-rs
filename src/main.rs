use components::{ask_network, GameState, ServerMode};

use std::{
    env,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
};

mod components;
mod game_logic;
mod http_server;
mod ws_server;

use game_logic::game_logic_loop;
use http_server::create_http_server;
use ws_server::create_ws_server;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let server_mode = if args.len() == 2 {
        if args[1] == "prod" {
            ServerMode::Production
        } else if args[1] == "dev" {
            ServerMode::Development
        } else {
            ServerMode::Ask
        }
    } else {
        ServerMode::Ask
    };

    let ip_addr: IpAddr = match server_mode {
        ServerMode::Development => "127.0.0.1".parse().unwrap(),
        ServerMode::Production => local_ip_address::local_ip().unwrap(),
        ServerMode::Ask => ask_network(),
    };

    let master_game_state = Arc::new(Mutex::new(GameState::new()));

    let http_server = thread::spawn(move || {
        create_http_server(ip_addr);
    });

    let ws_game_state_clone = master_game_state.clone();
    let ws_server = thread::spawn(move || {
        create_ws_server(ip_addr, ws_game_state_clone);
    });

    let logic_game_state_clone = master_game_state.clone();
    let game_logic_thread = thread::spawn(move || {
        game_logic_loop(logic_game_state_clone);
    });

    for thread in [http_server, ws_server, game_logic_thread] {
        thread.join().unwrap();
    }
}
