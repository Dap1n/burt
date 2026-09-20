use std::{
    mem,
    sync::{LazyLock, RwLock},
};

/// Global run state. Tracks whether BURT is idle, doing a manual recording,
/// or in a speedrun, plus the per-run data that only matters while a speedrun
/// is active.
pub enum RunState {
    Idle,
    ManualRec,
    Speedrun {
        folder: String,
        /// (edict_ptr, entity_ptr) of the completion entity marker.
        marker: Option<(usize, usize)>,
        no_vault_save: bool,
    },
}

pub static RUN_STATE: LazyLock<RwLock<RunState>> = LazyLock::new(|| RwLock::new(RunState::Idle));

/// Returns whether run state is not idle
pub fn is_recording() -> bool {
    !matches!(&*RUN_STATE.read().unwrap(), RunState::Idle)
}

pub fn current_folder() -> Option<String> {
    match &*RUN_STATE.read().unwrap() {
        RunState::Speedrun { folder, .. } => Some(folder.clone()),
        _ => None,
    }
}

/// Consumes the `no_vault` one-shot flag (sets it back to false).
/// Returns false if we're not in a speedrun.
pub fn take_no_vault() -> bool {
    if let RunState::Speedrun {
        no_vault_save: no_vault,
        ..
    } = &mut *RUN_STATE.write().unwrap()
    {
        mem::take(no_vault)
    } else {
        false
    }
}

pub fn set_marker(edict: usize, entity: usize) {
    if let RunState::Speedrun { marker, .. } = &mut *RUN_STATE.write().unwrap() {
        *marker = Some((edict, entity));
    }
}

pub fn clear_marker() {
    if let RunState::Speedrun { marker, .. } = &mut *RUN_STATE.write().unwrap() {
        *marker = None;
    }
}

pub fn get_marker() -> Option<(usize, usize)> {
    match &*RUN_STATE.read().unwrap() {
        RunState::Speedrun { marker, .. } => *marker,
        _ => None,
    }
}
