use std::sync::atomic::{AtomicBool, Ordering};

use portable_atomic::AtomicF32;

use crate::{
    clog, define_bounds,
    game::{
        hooks::{
            Subscriber,
            process_cmd::{self, ProcessCmdArgs, schedule_movement},
        },
        offsets_provider::provider,
    },
    structs::Color,
};
use burt_command::burt_command;

static ABSMOV_ACTIVE: AtomicBool = AtomicBool::new(false);
static ABSMOV_ANGLE: AtomicF32 = AtomicF32::new(0.0);
static ABSMOV_STRENGTH: AtomicF32 = AtomicF32::new(1.0);

fn on_process_cmd(args: &mut ProcessCmdArgs) {
    if !ABSMOV_ACTIVE.load(Ordering::Relaxed) || args.cmd.is_null() {
        return;
    }

    let world_angle = ABSMOV_ANGLE.load(Ordering::Relaxed).to_radians();
    let strength = ABSMOV_STRENGTH.load(Ordering::Relaxed);

    let cmd_yaw =
        unsafe { *(args.cmd.add(provider().cusercmd_yaw_off()) as *const f32) }.to_radians();

    let rel = world_angle - cmd_yaw;
    let local_forward = rel.cos();
    let local_side = -rel.sin();

    schedule_movement(local_side * strength, local_forward * strength);
}

pub fn enable_absmov(angle: f32, strength: f32) {
    process_cmd::subscribe(Subscriber::Absmov(on_process_cmd));
    ABSMOV_ANGLE.store(angle.rem_euclid(360.0), Ordering::Relaxed);
    ABSMOV_STRENGTH.store(strength, Ordering::Relaxed);
    ABSMOV_ACTIVE.store(true, Ordering::Relaxed);
}

pub fn disable_absmov() {
    if ABSMOV_ACTIVE.swap(false, Ordering::Relaxed) {
        ABSMOV_ANGLE.store(0.0, Ordering::Relaxed);
        ABSMOV_STRENGTH.store(1.0, Ordering::Relaxed);
        process_cmd::unsubscribe(Subscriber::Absmov(on_process_cmd));
    }
}

/// Absolute movement tool: moves toward a world direction regardless of where
/// the player is looking.
///
/// `angle:` degrees in world space (0 = +X, 90 = +Y, 180 = -X, 270 = -Y)
/// `strength:` value in range 0.0..1.0 that maximum player's speed 175 gets multiplied by.
///   By default it's 1.0
/// `enabled:` whether to enable absmov tool. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_absmov(angle: f32, strength: Option<f32>, enabled: Option<bool>) {
    let strength = strength.unwrap_or(1.0);
    define_bounds!(0.0 <= strength <= 1.0);

    let enabled = enabled.unwrap_or_else(|| {
        !ABSMOV_ACTIVE.load(Ordering::Relaxed)
            || angle != ABSMOV_ANGLE.load(Ordering::Relaxed)
            || strength != ABSMOV_STRENGTH.load(Ordering::Relaxed)
    });

    if enabled {
        enable_absmov(angle, strength);
        clog!(prefix;
            [Color::WHITE => "Absmov "],
            [Color::LIME => "enabled"],
        );
    } else {
        disable_absmov();
        clog!(prefix;
            [Color::WHITE => "Absmov "],
            [Color::RED => "disabled"],
        );
    }
}
