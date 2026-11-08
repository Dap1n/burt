use std::fs;

use crate::{
    cexec, clog,
    commands::run::state::{self, RunState},
    consts::SPEEDRUN_FOLDER_FORMAT,
    engine_build::{self, EngineBuild},
    log,
    os::format_local_time,
    structs::Color,
};
use burt_command::burt_command;

/// Starts the speedrun with automatic demo recording.
///
/// `use_vault_save:` whether to use vault save or start a fresh new game
///   - If not provided, tries to fetch `vault.sav`, and if not found, falls back to map load
///   - If new game is being started, intro video in July 2009 build will be automatically
///     skipped by emulating `enter` keypress
#[burt_command]
pub fn burt_sr_start(use_vault_save: Option<bool>) {
    if state::is_recording() {
        clog!(err_prefix; "Can't start the speedrun while recording");
        return;
    }

    let current_build = engine_build::current_build();

    let parent_name = if current_build == EngineBuild::Project2008 {
        "p2008"
    } else {
        "portal2"
    };

    let vault_save_exists: Option<bool> = use_vault_save.unwrap_or(true).then(|| {
        fs::read_dir(format!("{parent_name}/SAVE"))
            .ok()
            .map(|rd| {
                rd.filter_map(|x| x.ok())
                    .any(|x| x.file_name().to_str() == Some("vault.sav"))
            })
            .unwrap_or_default()
    });

    if use_vault_save.unwrap_or_default() && !vault_save_exists.unwrap() {
        clog!(err_prefix;
            [Color::RED => "vault.sav"],
            [Color::WHITE => " couldn't be found in "],
            [Color::RED => "SAVE"],
            [Color::WHITE => " directory"],
        );
        return;
    }

    let folder = format_local_time(SPEEDRUN_FOLDER_FORMAT).to_string().into();

    clog!(prefix;
        [Color::WHITE => "Recording to "],
        [Color::LIME => "{parent_name}/records/{folder}"],
    );
    log!("[sr_start] Recording demos to {folder}");

    let _ = fs::create_dir(format!("{parent_name}/records/{folder}"));

    let with_vault_save = vault_save_exists.unwrap_or_default();
    *state::RUN_STATE.write().unwrap() = RunState::Speedrun {
        folder,
        with_vault_save,
    };

    if with_vault_save {
        cexec!("load vault");
    } else if current_build == EngineBuild::Project2008 {
        cexec!("map lab_intro");
    } else {
        cexec!("map p2_lab_prehub_1");
    }
}
