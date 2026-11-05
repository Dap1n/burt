use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use portable_atomic::AtomicF32;

use crate::{
    clog,
    game::{
        hooks::{
            Subscriber,
            create_move::{self, CreateMoveArgs},
            process_cmd,
        },
        math::shortest_dyaw,
    },
    structs::{Color, Easing},
};
use burt_command::burt_command;

static SETANG_ACTIVE: AtomicBool = AtomicBool::new(false);

static START_PITCH: AtomicF32 = AtomicF32::new(0.0);
static START_YAW: AtomicF32 = AtomicF32::new(0.0);
static TARGET_PITCH: AtomicF32 = AtomicF32::new(0.0);
static TARGET_YAW: AtomicF32 = AtomicF32::new(0.0);
static DURATION_TICKS: AtomicU32 = AtomicU32::new(0);
static ELAPSED_TICKS: AtomicU32 = AtomicU32::new(0);
static EASING: AtomicU8 = AtomicU8::new(0);

fn on_create_move(_: &mut CreateMoveArgs) {
    if !SETANG_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    let duration = DURATION_TICKS.load(Ordering::Relaxed);
    let elapsed = ELAPSED_TICKS.fetch_add(1, Ordering::Relaxed);

    let progress = if duration == 0 {
        1.0
    } else {
        (elapsed as f32 / duration as f32).clamp(0.0, 1.0)
    };

    let start_pitch = START_PITCH.load(Ordering::Relaxed);
    let start_yaw = START_YAW.load(Ordering::Relaxed);
    let target_pitch = TARGET_PITCH.load(Ordering::Relaxed);
    let target_yaw = TARGET_YAW.load(Ordering::Relaxed);
    let easing = Easing::from_index(EASING.load(Ordering::Relaxed));
    let eased = easing.apply(progress);

    let interp_pitch = start_pitch + (target_pitch - start_pitch) * eased;
    let total_dyaw = shortest_dyaw(target_yaw - start_yaw);
    let interp_yaw = start_yaw + total_dyaw * eased;

    create_move::schedule_angles(interp_pitch, interp_yaw);

    if progress == 1.0 {
        disable_setang();
    }
}

pub fn enable_setang(pitch: f32, yaw: f32, duration: u32, easing: Easing) {
    let (start_pitch, start_yaw) = process_cmd::cached_body_angles();
    let target_yaw_norm = yaw.rem_euclid(360.0);

    START_PITCH.store(start_pitch, Ordering::Relaxed);
    START_YAW.store(start_yaw, Ordering::Relaxed);
    TARGET_PITCH.store(pitch, Ordering::Relaxed);
    TARGET_YAW.store(target_yaw_norm, Ordering::Relaxed);
    DURATION_TICKS.store(duration, Ordering::Relaxed);
    ELAPSED_TICKS.store(0, Ordering::Relaxed);
    EASING.store(easing.to_index(), Ordering::Relaxed);

    SETANG_ACTIVE.store(true, Ordering::Relaxed);
    create_move::subscribe(Subscriber::Setang(on_create_move));
}

pub fn disable_setang() {
    if SETANG_ACTIVE.swap(false, Ordering::Relaxed) {
        create_move::unsubscribe(Subscriber::Setang(on_create_move));
    }
}

/// Rotates the camera toward given pitch/yaw using mouse injection.
///
/// `pitch, yaw:` target angles
/// `time:` duration in game ticks. If omitted or `0`, aiming is instant
/// `easing:` either `linear`, `sin`, `cubic` or `exp`. Default is `linear`
#[burt_command(cheat)]
pub fn burt_setang(pitch: f32, yaw: f32, time: Option<u32>, easing: Option<Easing>) {
    let duration = time.unwrap_or_default();
    let easing = easing.unwrap_or_default();
    enable_setang(pitch, yaw, duration, easing);
    clog!(prefix;
        [Color::WHITE => "Setang "],
        [Color::LIME => "enabled"],
    );
}
