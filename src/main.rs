use components::{GameState, ServerMode};

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;

mod components;
mod firsttry;
mod http_server;
mod physics_server;
mod util;
mod variables;
mod ws_parse;
mod ws_server;

use http_server::create_http_server;
use physics_server::physics_loop;
use ws_server::create_ws_server;

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
        physics_loop(physics_game_state_clone);
    });

    threads.push(http_server);
    threads.push(ws_server);
    threads.push(physics_server);

    for thread in threads {
        thread.join().unwrap();
    }
}
