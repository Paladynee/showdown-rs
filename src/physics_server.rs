use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{components::GameState, variables};

pub fn physics_loop(game_state: Arc<Mutex<GameState>>) {
    #[expect(unused_assignments)]
    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1) / variables::PHYSICS_TICKRATE;

    loop {
        update_physics(&game_state);

        let now = Instant::now();
        last_update = now;
        if now - last_update < update_interval {
            let sleep_duration = update_interval - (now - last_update);
            thread::sleep(sleep_duration);
        }
    }
}

fn update_physics(_game_state: &Arc<Mutex<GameState>>) {
    // for now, no physics. in the future, bullets will go according to their velocity etc.
}
