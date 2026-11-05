use crate::{
    cexec, clog,
    commands::run::state::{self, RunState},
    log,
    structs::Color,
};

use burt_command::burt_command;

pub fn stop_internal(automatic: bool) {
    let folder_name = {
        let mut st = state::RUN_STATE.write().unwrap();
        let RunState::Speedrun { folder, .. } = &*st else {
            clog!(prefix; "Speedrun wasn't started");
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

/// Ends the speedrun.
/// Gets automatically invoked when game ends.
#[burt_command]
pub fn burt_sr_stop() {
    stop_internal(false);
}
