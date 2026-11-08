use std::sync::atomic::{AtomicI32, Ordering};

use crate::{
    bridge::vtable::is_simulating,
    commands::run::{rec, state},
    engine_build::{self, EngineBuild},
    game::engine_input,
    vtable::level_init::IN_LOAD,
};

static SKIP_COUNTDOWN: AtomicI32 = AtomicI32::new(0);

fn tick_cutscene_skip() {
    if !is_simulating() {
        return;
    }

    let remaining = SKIP_COUNTDOWN.load(Ordering::Relaxed);
    if remaining <= 0 {
        return;
    }

    let next = remaining - 1;
    SKIP_COUNTDOWN.store(next, Ordering::Relaxed);

    match next {
        55 | 50 | 45 | 40 | 30 | 10 | 2 => unsafe { engine_input::press_button(64) },
        53 | 48 | 43 | 38 | 28 | 8 | 0 => unsafe { engine_input::release_button(64) },
        _ => (),
    }
}

pub fn game_frame() {
    tick_cutscene_skip();

    if !IN_LOAD.swap(false, Ordering::Relaxed) || !state::is_recording() {
        return;
    }

    if engine_build::current_build() == EngineBuild::July2009 {
        if state::no_vault() {
            SKIP_COUNTDOWN.store(70, Ordering::Relaxed);
        } else {
            SKIP_COUNTDOWN.store(0, Ordering::Relaxed);
        }
    }

    let map = rec::CURRENT_MAP.read().unwrap();
    rec::on_map_ready(map.as_str());
}
