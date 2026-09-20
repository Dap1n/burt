use std::{
    borrow::Cow,
    sync::{
        LazyLock, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::Local;

use crate::{
    bridge::commands::CCommand,
    cexec, clog,
    commands::run_state::{self, RunState},
    consts::RECORDS_FORMAT,
    log,
};
use burt_macros::burt_command;

pub static CURRENT_MAP: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));
pub static IN_LOAD: AtomicBool = AtomicBool::new(false);

fn sanitize(map_name: &str) -> Cow<'_, str> {
    if map_name.is_empty() {
        Cow::Borrowed("unknown")
    } else {
        Cow::Owned(
            map_name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || ['_', '-'].contains(&c) {
                        c
                    } else {
                        '_'
                    }
                })
                .collect(),
        )
    }
}

fn get_filename(map_name: &str) -> String {
    let sanitized = sanitize(map_name);
    let ts = Local::now().format(RECORDS_FORMAT);
    format!("burt_{sanitized}_{ts}")
}

pub fn on_map_ready(map_name: &str) {
    let filename = get_filename(map_name);
    cexec!("stop");
    if let Some(folder) = run_state::current_folder() {
        cexec!("record records/{folder}/{filename}");
    } else {
        cexec!("record records/{filename}");
    }
}

/// Toggle automatic demo recording on level load
#[burt_command]
pub fn burt_rec(enabled: bool) {
    {
        let mut st = run_state::RUN_STATE.write().unwrap();
        match (&*st, enabled) {
            (RunState::Speedrun { .. }, _) => {
                clog!("Can not use the command while in speedrun");
                return;
            }
            (RunState::ManualRec, true) => {
                clog!("Already recording");
                return;
            }
            (RunState::Idle, false) => {
                clog!("Was not recording already");
                return;
            }
            (RunState::Idle, true) => {
                *st = RunState::ManualRec;
            }
            (RunState::ManualRec, false) => {
                *st = RunState::Idle;
            }
        }
    }

    if enabled {
        let map = CURRENT_MAP.read().unwrap().clone();
        on_map_ready(&map);
        clog!(prefix; "Recording started");
        log!("[burt_rec] Recording started");
    } else {
        cexec!("stop");
        IN_LOAD.store(false, Ordering::Relaxed);
        clog!(prefix; "Recording stopped");
    }
}
