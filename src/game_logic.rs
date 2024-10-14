use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::components::{GameState, PHYSICS_TICKRATE};

pub fn game_logic_loop(game_state: Arc<Mutex<GameState>>) {
    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1) / PHYSICS_TICKRATE;

    let mut dead_players = vec![];

    loop {
        let dt = last_update.elapsed().as_secs_f32();
        update_logic(&game_state, dt, &mut dead_players);

        let now = Instant::now();
        last_update = now;
        if now - last_update < update_interval {
            let sleep_duration = update_interval - (now - last_update);
            thread::sleep(sleep_duration);
        }
    }
}

fn update_logic(
    game_state: &Arc<Mutex<GameState>>,
    dt: f32,
    dead_players: &mut Vec<(String, f32)>,
) {
    let mut lock = game_state.lock().unwrap();

    // strip bullets that are out of life
    lock.bullets.retain(|bullet| bullet.life > 0.0);

    // update bullet life by dt
    for bullet in &mut lock.bullets {
        bullet.life -= dt;
    }

    // update bullet positions by vel * dt
    for bullet in &mut lock.bullets {
        bullet.x += bullet.velx * dt;
        bullet.y += bullet.vely * dt;
    }

    // check if any bullet is colliding with any player
    let player_locations = lock.players.clone();
    let mut players_hit = vec![];
    for bullet in &mut lock.bullets {
        for (player_id, player) in &player_locations {
            if player_id == &bullet.owner {
                continue;
            }
            let player_location = (player.x, player.y);
            let bullet_location = (bullet.x, bullet.y);
            if is_colliding(player_location, bullet_location) {
                players_hit.push(player_id.to_owned());
                bullet.life = 0.0;
            }
        }
    }

    // update player hp
    for player_id in players_hit {
        if let Some(player) = lock.players.get_mut(&player_id) {
            if player.take_damage(10.0) {
                dead_players.push((player_id, 5.0));
                player.x = 2147483647.0;
                player.y = 2147483647.0;
            };
        }
    }

    // update player respawn
    dead_players.retain_mut(|(player_id, respawn_time)| {
        *respawn_time -= dt;
        if *respawn_time <= 0.0 {
            if let Some(player) = lock.players.get_mut(player_id) {
                player.x = 50.0;
                player.y = 50.0;
            }
            false
        } else {
            true
        }
    });
}

fn is_colliding(player_location: (f32, f32), bullet_location: (f32, f32)) -> bool {
    let dx = player_location.0 - bullet_location.0;
    let dy = player_location.1 - bullet_location.1;
    let distance = (dx * dx + dy * dy).sqrt();

    distance < 25.0
}
