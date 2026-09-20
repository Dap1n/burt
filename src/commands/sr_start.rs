use std::fs;

use crate::{
    bridge::{commands::CCommand, console::Color},
    cexec, clog,
    commands::run_state::{self, RunState},
    consts::SPEEDRUN_FOLDER_FORMAT,
    log,
};
use burt_macros::burt_command;
use chrono::Local;

/// Starts the speedrun with automatic demo recording.
/// If `use_vault_save` is set to `true`, loads `vault.sav`.
/// If set to `false`, starts fresh new game.
/// If not provided, tries to fetch `vault.sav`, and if not found, falls back to map load.
#[burt_command]
pub fn burt_sr_start(use_vault_save: Option<bool>) {
    if run_state::is_recording() {
        clog!("Can not start the speedrun while recording");
        return;
    }

    let vault_save_exists = use_vault_save.unwrap_or(true).then(|| {
        fs::read_dir("portal2/SAVE")
            .ok()
            .map(|rd| {
                rd.filter_map(|x| x.ok())
                    .any(|x| x.file_name().to_str() == Some("vault.sav"))
            })
            .unwrap_or_default()
    });

    if use_vault_save.unwrap_or_default() && !vault_save_exists.unwrap() {
        clog!(
            [Color::RED => "vault.sav"],
            [Color::WHITE => " could not be found in "],
            [Color::RED => "SAVE"],
            [Color::WHITE => " directory"],
        );
        return;
    }

    let folder = Local::now().format(SPEEDRUN_FOLDER_FORMAT).to_string();

    clog!(prefix;
        [Color::WHITE => "Recording to "],
        [Color::LIME => "portal2/records/{folder}"],
    );
    log!("[sr_start] Recording demos to {folder}");

    let _ = fs::create_dir(format!("portal2/records/{folder}"));

    let no_vault_save = !vault_save_exists.unwrap_or_default();
    *run_state::RUN_STATE.write().unwrap() = RunState::Speedrun {
        folder,
        marker: None,
        no_vault_save,
    };

    if no_vault_save {
        cexec!("map p2_lab_prehub_1");
    } else {
        cexec!("load vault");
    }
}
