use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use portable_atomic::AtomicF32;

use crate::{clog, define_bounds};
use burt_command::burt_command;

pub static BVAR_TAS_CHECK_DISABLE: AtomicBool = AtomicBool::new(false);
pub static BVAR_TAS_CHECK_MAX_REPLAYS: AtomicU32 = AtomicU32::new(15);
pub static BVAR_TAS_COMMANDS_DISABLE: AtomicBool = AtomicBool::new(false);
pub static BVAR_TAS_INTERPOLATE: AtomicBool = AtomicBool::new(false);
pub static BVAR_TAS_PAUSE_AT: AtomicU32 = AtomicU32::new(0);
pub static BVAR_TAS_PLAYBACK_RATE: AtomicF32 = AtomicF32::new(1.0);
pub static BVAR_TAS_SKIP_TO: AtomicU32 = AtomicU32::new(0);
pub static BVAR_TAS_TOOLS_DISABLE: AtomicBool = AtomicBool::new(false);

/// Whether to globally disable the TAS check tool.
#[burt_command(var = BVAR_TAS_CHECK_DISABLE)]
pub fn bvar_tas_check_disable(disable: bool) {
    BVAR_TAS_CHECK_DISABLE.store(disable, Ordering::Relaxed);
}

/// Defines how many times the check tool will restart playback
/// until it gives up and stops the TAS.
///
/// If set to `0`, will never give up.
#[burt_command(var = BVAR_TAS_CHECK_MAX_REPLAYS)]
pub fn bvar_tas_check_max_replays(replays: u32) {
    BVAR_TAS_CHECK_MAX_REPLAYS.store(replays, Ordering::Relaxed);
}

/// Whether to globally disable commands in TAS script.
#[burt_command(var = BVAR_TAS_COMMANDS_DISABLE)]
pub fn bvar_tas_commands_disable(disable: bool) {
    BVAR_TAS_COMMANDS_DISABLE.store(disable, Ordering::Relaxed);
}

/// Whether to preserve client interpolation in TAS playback.
#[burt_command(var = BVAR_TAS_INTERPOLATE)]
pub fn bvar_tas_interpolate(interpolate: bool) {
    BVAR_TAS_INTERPOLATE.store(interpolate, Ordering::Relaxed);
}

/// Automatically pauses the TAS at the given tick.
/// `0` disables pausing.
#[burt_command(var = BVAR_TAS_PAUSE_AT)]
pub fn bvar_tas_pause_at(tick: u32) {
    if tick != 0 && tick <= 12 {
        clog!(err_prefix;
            "Values in range 1-12 makes the game freeze, so you can't set them"
        );
    } else {
        BVAR_TAS_PAUSE_AT.store(tick, Ordering::Relaxed);
    }
}

/// The rate at which to play back TAS scripts.
///
/// If set to `0` or a sufficiently large value, the rate will be uncapped
/// and determined by your current FPS.
#[burt_command(var = BVAR_TAS_PLAYBACK_RATE)]
pub fn bvar_tas_playback_rate(rate: f32) {
    define_bounds!(0.0 <= rate <= 10_000.0);
    BVAR_TAS_PLAYBACK_RATE.store(rate, Ordering::Relaxed);
}

/// Quickly forwards the TAS up to the given tick.
/// If set to `0`, fast-forward is disabled.
#[burt_command(var = BVAR_TAS_SKIP_TO)]
pub fn bvar_tas_skip_to(tick: u32) {
    BVAR_TAS_SKIP_TO.store(tick, Ordering::Relaxed);
}

/// Whether to globally disable all TAS tools.
#[burt_command(var = BVAR_TAS_TOOLS_DISABLE)]
pub fn bvar_tas_tools_disable(enabled: bool) {
    BVAR_TAS_TOOLS_DISABLE.store(enabled, Ordering::Relaxed);
}
