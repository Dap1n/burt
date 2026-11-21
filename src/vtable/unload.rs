use std::sync::atomic::Ordering;

use crate::{bridge::commands, commands::tas, game::hooks, log};

pub fn unload() {
    log!("Unload called");
    if tas::SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        tas::end_tas();
    }
    hooks::uninstall_all().expect("Failed to uninstall hooks");
    commands::unregister_plugin_commands();
}
