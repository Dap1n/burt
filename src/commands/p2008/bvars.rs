use std::sync::atomic::{AtomicI8, Ordering};

use burt_command::burt_command;

use crate::define_bounds;

pub static BVAR_CAPTURE_OVERRIDE: AtomicI8 = AtomicI8::new(-1);

/// Overrides the ability to capture certain objects.
///
/// `-1` = no override (you can only capture what the game allows)
/// `0` = disable capturing entirely
/// `1` = capture anything
/// `2` = invert the capturing ability (you can only capture what the game disallows)
#[burt_command(var = BVAR_CAPTURE_OVERRIDE)]
pub fn bvar_capture_override(capture_override: i8) {
    define_bounds!(-1 <= capture_override <= 2);
    BVAR_CAPTURE_OVERRIDE.store(capture_override, Ordering::Relaxed);
}
