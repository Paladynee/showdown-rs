use std::{
    net::{IpAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use tungstenite::{Message, WebSocket};

use crate::components::{GameState, GameStatePacket, Player, WebSocketClientData};
use crate::components::{WS_PORT, WS_TICKRATE};
// -------------------------------------- WS SERVER --------------------------------------
pub fn create_ws_server(ip_addr: IpAddr, game_state_mutex: Arc<Mutex<GameState>>) {
    let port = WS_PORT;

    let ws_listener = TcpListener::bind((ip_addr, port)).unwrap();
    eprintln!("[THREAD::WS] ws://{}:{} is listening...", ip_addr, port);

    let mut connection_threads = vec![];

    for stream in ws_listener.incoming().flatten() {
        let game_state_mutex = game_state_mutex.clone();
        let handle = thread::spawn(move || {
            handle_websocket_handshake(stream, game_state_mutex);
        });

        connection_threads.push(handle);
    }

    for thread in connection_threads {
        thread.join().unwrap();
    }
}

// -------------------------------------- WS HANDSHAKE --------------------------------------
fn handle_websocket_handshake(stream: TcpStream, game_state_mutex: Arc<Mutex<GameState>>) {
    eprintln!(
        "[THREAD::WS] {}: new websocket connection",
        stream.peer_addr().unwrap()
    );

    let ws_identifier = stream.peer_addr().unwrap().to_string();
    let handshake_result = tungstenite::accept(stream);

    match handshake_result {
        Err(_) => {
            eprintln!("[THREAD::WS] {}: handshake failed", ws_identifier);
        }

        Ok(websocket) => handle_websocket_connection(websocket, ws_identifier, game_state_mutex),
    }
}

// -------------------------------------- WS CONNECTION --------------------------------------
fn handle_websocket_connection(
    mut websocket: WebSocket<TcpStream>,
    ws_identifier: String,
    game_state_mutex: Arc<Mutex<GameState>>,
) {
    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1) / WS_TICKRATE;

    game_state_mutex
        .lock()
        .unwrap()
        .players
        .insert(ws_identifier.clone(), Player::default());

    loop {
        if websocket_tick(
            &mut websocket,
            &ws_identifier,
            &mut last_update,
            update_interval,
            &game_state_mutex,
        ) {
            break;
        };
    }
}

// -------------------------------------- WS TICK --------------------------------------
fn websocket_tick(
    websocket: &mut WebSocket<TcpStream>,
    ws_identifier: &str,
    last_update: &mut Instant,
    update_interval: Duration,
    game_state_mutex: &Arc<Mutex<GameState>>,
) -> bool {
    // read incoming message from client
    match websocket.read() {
        Err(_) => {
            eprintln!("[THREAD::WS] {}: read failed", ws_identifier);
            return true;
        }

        Ok(received_message) => {
            if received_message.is_close() {
                eprintln!("[THREAD::WS] {}: received close message", ws_identifier);
                game_state_mutex
                    .lock()
                    .unwrap()
                    .players
                    .remove(ws_identifier);
                return true;
            }

            if received_message.is_text() {
                let text = received_message.into_text().unwrap();
                if let Some(client_data) = parse_websocket_message(&text) {
                    update_game_state(client_data, ws_identifier, game_state_mutex);
                } else {
                    eprintln!("[THREAD::WS] {}: failed to parse message", ws_identifier);
                };
            } else {
                eprintln!("[THREAD::WS] {}: received non-text message", ws_identifier);
                return false;
            }
        }
    }

    // send updated gamestate to client
    let now = Instant::now();
    if now.duration_since(*last_update) >= update_interval {
        let game_state = {
            let lock = game_state_mutex.lock().unwrap();
            (*lock).clone()
        };

        let constructed_packet = GameStatePacket::new(game_state, ws_identifier);
        let serialized_state = serde_json::to_string(&constructed_packet).unwrap();
        let write_result = websocket.write(Message::Text(serialized_state));
        if write_result.is_err() {
            eprintln!("[THREAD::WS] {}: write failed", ws_identifier);
            return true;
        }
        let flush_result = websocket.flush();
        if flush_result.is_err() {
            eprintln!("[THREAD::WS] {}: flush failed", ws_identifier);
            return true;
        }

        *last_update = now;
    }

    false
}

pub fn parse_websocket_message(message: &str) -> Option<WebSocketClientData> {
    let parsed = serde_json::from_str(message);
    // if let Ok(msg) = &parsed {
    //     eprintln!("Recieved message: {:#?}", msg);
    // }
    parsed.ok()
}

// -------------------------------------- GAMESTATE UPDATE --------------------------------------
fn update_game_state(
    client_data: WebSocketClientData,
    ws_identifier: &str,
    game_state_mutex: &Arc<Mutex<GameState>>,
) {
    let mut game_state = game_state_mutex.lock().unwrap();

    game_state.bullets.extend(client_data.new_bullets);
    let player = game_state.players.get_mut(ws_identifier);

    if let Some(player) = player {
        // before updating the player, check if the client_data is teleporting the player around
        let distance = ((player.x - client_data.player.x).powi(2)
            + (player.y - client_data.player.y).powi(2))
        .sqrt();
        if distance <= 50.0 {
            player.x = client_data.player.x;
            player.y = client_data.player.y;
        }
    }
}
