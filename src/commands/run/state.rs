use std::sync::{LazyLock, RwLock};

/// Global run state. Tracks whether BURT is idle, doing a manual recording,
/// or in a speedrun, plus the per-run data that only matters while a speedrun
/// is active.
#[derive(Debug, Clone)]
pub enum RunState {
    Idle,
    ManualRec,
    Speedrun {
        folder: Box<str>,
        with_vault_save: bool,
    },
}

pub static RUN_STATE: LazyLock<RwLock<RunState>> = LazyLock::new(|| RwLock::new(RunState::Idle));

/// Returns whether run state is not idle
pub fn is_recording() -> bool {
    !matches!(&*RUN_STATE.read().unwrap(), RunState::Idle)
}

/// Returns whether in speedrun
pub fn in_speedrun() -> bool {
    matches!(&*RUN_STATE.read().unwrap(), RunState::Speedrun { .. })
}

pub fn current_folder() -> Option<Box<str>> {
    match &*RUN_STATE.read().unwrap() {
        RunState::Speedrun { folder, .. } => Some(folder.clone()),
        _ => None,
    }
}

pub fn no_vault() -> bool {
    matches!(
        *RUN_STATE.read().unwrap(),
        RunState::Speedrun {
            with_vault_save: false,
            ..
        }
    )
}
