use std::{
    borrow::Cow,
    sync::{LazyLock, RwLock},
};

use crate::{
    cexec, clog,
    commands::run::state::{self, RunState},
    consts::RECORDS_FORMAT,
    log,
    os::format_local_time,
};
use burt_command::burt_command;

pub static CURRENT_MAP: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));

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
    let ts = format_local_time(RECORDS_FORMAT);
    format!("burt_{sanitized}_{ts}")
}

pub fn on_map_ready(map_name: &str) {
    let filename = get_filename(map_name);
    cexec!("stop");
    if let Some(folder) = state::current_folder() {
        cexec!("record records/{folder}/{filename}");
    } else {
        cexec!("record records/{filename}");
    }
}

/// Toggles automatic demo recording on level load.
///
/// `enabled:` whether to enable recording. By default flips automatically
#[burt_command]
pub fn burt_rec(enabled: Option<bool>) {
    let enabled =
        enabled.unwrap_or_else(|| matches!(&*state::RUN_STATE.read().unwrap(), RunState::Idle));

    let mut st = state::RUN_STATE.write().unwrap();
    match (&*st, enabled) {
        (RunState::Speedrun { .. }, _) => {
            clog!(err_prefix; "Can't use the command while in speedrun");
            return;
        }
        (RunState::ManualRec, true) => {
            clog!(err_prefix; "Already recording");
            return;
        }
        (RunState::Idle, false) => {
            clog!(prefix; "Wasn't recording already");
            return;
        }
        (RunState::Idle, true) => {
            *st = RunState::ManualRec;
        }
        (RunState::ManualRec, false) => {
            *st = RunState::Idle;
        }
    }
    drop(st);

    if enabled {
        let map = CURRENT_MAP.read().unwrap().clone();
        on_map_ready(&map);
        clog!(prefix; "Recording started");
        log!("[burt_rec] Recording started");
    } else {
        cexec!("stop");
        clog!(prefix; "Recording stopped");
        log!("[burt_rec] Recording stopped");
    }
}
