use std::sync::atomic::Ordering;

use crate::{
    bridge::{commands::CCommand, console::Color},
    cexec, clog,
    commands::{
        rec::IN_LOAD,
        run_state::{self, RunState},
    },
    log,
};
use burt_macros::burt_command;

pub fn stop_internal(automatic: bool) {
    let folder_name = {
        let mut st = run_state::RUN_STATE.write().unwrap();
        let RunState::Speedrun { folder, .. } = &*st else {
            clog!("Speedrun was not even started");
            return;
        };
        let folder = folder.clone();
        *st = RunState::Idle;
        folder
    };

    if automatic {
        log!("Automatically ending the speedrun");
    } else {
        log!("[sr_stop] Ending the speedrun");
    }

    IN_LOAD.store(false, Ordering::Relaxed);
    cexec!("stop");

    clog!(prefix;
        [Color::WHITE => "{}", if automatic {
            "Ending screen was reached, speedrun is over. Recorded to "
        } else {
            "Speedrun is stopped. Recorded to "
        }],
        [Color::LIME => "portal2/records/{folder_name}"],
    );
}

/// Ends the speedrun
#[burt_command]
pub fn burt_sr_stop() {
    stop_internal(false);
}
