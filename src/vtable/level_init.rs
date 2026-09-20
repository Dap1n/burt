use std::sync::atomic::Ordering;

use crate::commands::{
    rec::{CURRENT_MAP, IN_LOAD},
    run_state,
};

pub fn level_init(map_name: &str) {
    *CURRENT_MAP.write().unwrap() = map_name.to_string();

    if run_state::is_recording() {
        IN_LOAD.store(true, Ordering::Relaxed);
    }
}
