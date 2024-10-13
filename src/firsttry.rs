use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{net, thread};
use tungstenite::{accept, Message};

use crate::http_server::get_content_type;


const SERVER_TICKRATE: u32 = 2;

#[derive(Serialize, Deserialize, Debug)]
struct GameState {
    players: HashMap<String, Player>,
    bullets: Vec<Bullet>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Bullet {
    x: f32,
    y: f32,
    velx: f32,
    vely: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
    hp: f32,
    x: f32,
    y: f32,
}

const IPADDR: &str = "127.0.0.1";
const HTTP_PORT: &str = "8080";
const WS_PORT: &str = "8081";

fn old_main() {
    // let local_ip = local_ip_address::local_ip().unwrap();
    let local_ip = IPADDR;
    let mut game_state = Arc::new(Mutex::new(GameState {
        players: HashMap::new(),
        bullets: Vec::new(),
    }));

    println!("Local IP: {}", local_ip);

    let base_dir = Path::new("./simple2Dgame/").canonicalize().unwrap();
    let mut handles = vec![];
    let http_handle = thread::spawn(move || {
        let http_listener =
            TcpListener::bind((local_ip, HTTP_PORT.parse::<u16>().expect("Infallible"))).unwrap();
        eprintln!("HTTP server listening on port 8080...");
        let mut thread_handles = vec![];

        for mut stream in http_listener.incoming().flatten() {
            let base_dir = base_dir.clone();
            let handle = thread::spawn(move || {
                eprintln!("Connection from {}", stream.peer_addr().unwrap());

                let mut buffer = [0; 4096];
                let read_amount = stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..read_amount]);

                // eprintln!("len: {}, request:\n{}", read_amount, request);

                handle_http_request(&mut stream, &request, &base_dir);
            });

            thread_handles.push(handle);
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }
    });

    let game_state_clone = game_state.clone();
    let ws_handle = thread::spawn(move || {
        let ws_listener =
            TcpListener::bind((local_ip, WS_PORT.parse::<u16>().expect("Infallible"))).unwrap();

        eprintln!("WebSocket server listening on port 8081...");

        let mut thread_handles = vec![];

        for mut stream in ws_listener.incoming().flatten() {
            let game_state_clone_clone = game_state_clone.clone();
            let handle = thread::spawn(move || {
                handle_websocket_request(&mut stream, game_state_clone_clone);
            });

            thread_handles.push(handle);
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }
    });

    let server_physics_thread_handle = thread::spawn(move || {
        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs(1) / SERVER_TICKRATE;

        loop {
            let now = Instant::now();
            if now.duration_since(last_update) >= update_interval {
                let mut state = game_state.lock().unwrap();
                for bullet in state.bullets.iter_mut() {
                    bullet.x += bullet.velx;
                    bullet.y += bullet.vely;
                }

                last_update = now;
            }

            // check bullet collision with players
            {
                let mut state = game_state.lock().unwrap();
                let mut bullets_to_remove = vec![];
                let mut players_to_take_damage = vec![];
                for (player_id, player) in state.players.iter() {
                    for bullet in state.bullets.iter() {
                        let dx = player.x - bullet.x;
                        let dy = player.y - bullet.y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < 50.0 {
                            players_to_take_damage.push(player_id.clone());
                            bullets_to_remove.push(bullet.clone());
                        }
                    }
                }

                state
                    .bullets
                    .retain(|bullet| !bullets_to_remove.contains(bullet));

                for player_id in players_to_take_damage {
                    if let Some(player) = state.players.get_mut(&player_id) {
                        player.hp -= 10.0;
                    }
                }
            }

            let now = Instant::now();
            let sleep_duration = update_interval
                .checked_sub(now.duration_since(last_update))
                .unwrap_or(Duration::from_secs(0));

            thread::sleep(sleep_duration);
        }
    });

    // let logger_handle = thread::spawn(move || loop {
    //     let state = game_state.lock().unwrap();
    //     eprintln!(
    //         "[SERVER] Connected players: {}\r\nPlayer List:\t{}",
    //         state.players.len(),
    //         // here we print every players player id
    //         state
    //             .players
    //             .keys()
    //             .map(|x| x.to_string())
    //             .collect::<Vec<String>>()
    //             .join("\r\n\t")
    //     );
    //     thread::sleep(Duration::from_secs(1));
    // });

    handles.push(http_handle);
    handles.push(ws_handle);
    // handles.push(logger_handle);
    handles.push(server_physics_thread_handle);

    for handle in handles {
        handle.join().unwrap();
    }
}

fn handle_websocket_request(stream: &mut net::TcpStream, game_state: Arc<Mutex<GameState>>) {
    eprintln!("[WSREQ] Handling websocket connection...");
    let player_id = stream.peer_addr().unwrap().to_string();
    let ws_result = accept(stream);

    {
        let mut state = game_state.lock().unwrap();
        state.players.insert(
            player_id.clone(),
            Player {
                x: 0.0,
                y: 0.0,
                hp: 100.0,
            },
        );
    }

    eprintln!("[WS] Player {} connected.", player_id);

    match ws_result {
        Ok(mut websocket) => {
            let mut last_update = Instant::now();
            let update_interval = Duration::from_secs(1) / SERVER_TICKRATE;

            // send the player their own player id so they know who they are
            let player_id_msg = format!("{{\"your_identifier\":\"{}\"}}", player_id);
            websocket.write(Message::Text(player_id_msg)).unwrap();
            websocket.flush().unwrap();

            loop {
                if let Ok(msg) = websocket.read() {
                    if msg.is_text() {
                        let text = msg.to_text().unwrap();
                        // eprintln!("[WS] Received a message from player {}: {}", player_id, msg);

                        if let Some(received_game_state) = parse_websocket_message(text) {
                            let mut state = game_state.lock().unwrap();
                            if let Some(player) = state.players.get_mut(&player_id) {
                                player.x = received_game_state.player.x;
                                player.y = received_game_state.player.y;
                                player.hp = received_game_state.player.hp;
                            }

                            state.bullets.extend(received_game_state.new_bullets);
                        }

                        // if let Some((new_x, new_y, new_hp)) = parse_websocket_message(text) {
                        //     let mut state = game_state.lock().unwrap();
                        //     if let Some(player) = state.players.get_mut(&player_id) {
                        //         player.x = new_x;
                        //         player.y = new_y;
                        //         player.hp = new_hp;
                        //     }
                        // } else {
                        //     eprintln!("[WS] Invalid message from player {}: {}", player_id, text);
                        // }
                    }

                    if msg.is_close() {
                        // remove player from game state
                        let mut state = game_state.lock().unwrap();
                        state.players.remove(&player_id);

                        eprintln!("[WS] Player {} disconnected", player_id);
                        break;
                    }
                }

                let now = Instant::now();
                if now.duration_since(last_update) >= update_interval {
                    let state = game_state.lock().unwrap();
                    let serialized_state = serde_json::to_string(&*state).unwrap();
                    websocket.write(Message::Text(serialized_state)).unwrap();
                    websocket.flush().unwrap();

                    last_update = now;
                }

                let now = Instant::now();
                let sleep_duration = update_interval
                    .checked_sub(now.duration_since(last_update))
                    .unwrap_or(Duration::from_secs(0));

                thread::sleep(sleep_duration);
            }
        }
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketReceivedGameData {
    player: Player,
    new_bullets: Vec<Bullet>,
}

fn parse_websocket_message(msg: &str) -> Option<WebSocketReceivedGameData> {
    // sample message:
    // [WS] Received a message from player 127.0.0.1:58800: {player: {x: 0.0, y: 0.0}, new_bullets: [{x: 0.0, y: 0.0, velx: 0.0, vely: 0.0}]}
    // lets use serde deserialize here to construct a WebSocketReceivedGameData
    let msg = serde_json::from_str(msg);
    if let Ok(msg) = &msg {
        eprintln!("Recieved message: {:#?}", msg);
    }
    msg.ok()
}

fn handle_http_request(stream: &mut net::TcpStream, request: &str, base_dir: &Path) {
    let mut path = match request.split_whitespace().nth(1) {
        Some(path) => path.to_string(),

        _ => {
            return;
        }
    };

    if path.starts_with("/") {
        path = path.replacen("/", "", 1);
    }

    let requested_path = base_dir.join(path);

    // eprintln!("Requested path: {}", requested_path.display());
    match requested_path.canonicalize() {
        Ok(np) => {
            if np.starts_with(base_dir) {
                let file = File::open(&np);
                match file {
                    Ok(f) => {
                        // eprintln!("Serving {}", np.display());
                        let mut reader = BufReader::new(&f);
                        let mut writer = BufWriter::new(stream);

                        let file_size = f.metadata().unwrap().len();
                        let content_type = get_content_type(&np);

                        // Create HTTP response headers
                        let response_headers = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                            content_type, file_size
                        );

                        writer.write_all(response_headers.as_bytes()).unwrap();

                        let mut buffer = [0; 1024];
                        loop {
                            let bytes_read = reader.read(&mut buffer).unwrap();
                            if bytes_read == 0 {
                                break;
                            }
                            writer.write_all(&buffer[..bytes_read]).unwrap();
                        }
                        writer.flush().unwrap();
                    }
                    Err(_) => {
                        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }
            }
        }
        Err(_) => {
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
