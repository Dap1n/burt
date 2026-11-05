use std::fs;

use crate::{clog, log, structs::Color};
use burt_command::burt_command;

/// Removes all `.mdmp` files in the game directory.
/// It's recommended to put it in `autoexec.cfg`.
#[burt_command]
pub fn burt_clear_mdmp() {
    let Ok(dir) = fs::read_dir(".") else {
        clog!(err_prefix; "Failed to read game directory");
        return;
    };

    let removed = dir
        .filter_map(|x| x.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|x| x.ends_with(".mdmp"))
        })
        .filter(|entry| fs::remove_file(entry.path()).is_ok())
        .count();

    log!("[clear_mdmp] removed={removed:?}");

    if removed == 0 {
        clog!(prefix;
            [Color::WHITE => "No "],
            [Color::YELLOW => ".mdmp"],
            [Color::WHITE => " files were removed"],
        );
        return;
    }

    clog!(prefix;
        [Color::WHITE => "Removed "],
        [Color::YELLOW => "{} .mdmp", removed],
        [Color::WHITE => "{}", if removed == 1 { " file" } else { " files" }],
    );
}
