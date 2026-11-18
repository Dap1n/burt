use std::{
    f32::consts::PI,
    sync::atomic::{AtomicBool, Ordering},
};

use portable_atomic::AtomicF32;

use crate::{
    clog,
    commands::strafe::math::{
        CvarCache, PlayerSnapshot, create_wish_dir, get_max_accel, ground_friction_velocity,
        length2d, predict_will_be_grounded,
    },
    game::{
        hooks::{
            Subscriber,
            process_cmd::{self, ProcessCmdArgs, schedule_movement},
        },
        player,
        provider::provider,
    },
    structs::Color,
};
use burt_command::burt_command;

static DECEL_ACTIVE: AtomicBool = AtomicBool::new(false);
static TARGET_SPEED: AtomicF32 = AtomicF32::new(0.0);

fn on_process_cmd(args: &mut ProcessCmdArgs) {
    if !DECEL_ACTIVE.load(Ordering::Relaxed) || args.cmd.is_null() {
        return;
    }

    let target_vel = TARGET_SPEED.load(Ordering::Relaxed);
    let Some(velocity) = player::velocity() else {
        return;
    };

    let (cur_pitch, cur_yaw) = process_cmd::cached_body_angles();

    let cv = CvarCache::load();
    let will_be_grounded = predict_will_be_grounded(velocity);

    let snapshot = PlayerSnapshot {
        velocity,
        pitch: cur_pitch,
        yaw: cur_yaw,
        will_be_grounded,
        ducked: player::is_ducked(),
        max_speed: cv.maxspeed,
        surface_friction: 1.0,
    };

    let tick_interval = provider().tickrate();
    let friction_vel = ground_friction_velocity(&snapshot, &cv, tick_interval);
    let player_vel = length2d(friction_vel);

    if target_vel >= player_vel {
        DECEL_ACTIVE.store(false, Ordering::Relaxed);
        process_cmd::unsubscribe(Subscriber::Decel(on_process_cmd));
        return;
    }

    let vel_yaw = friction_vel.1.atan2(friction_vel.0);
    let rel = vel_yaw + PI - cur_yaw.to_radians();

    let mut forward = rel.cos();
    let mut side = -rel.sin();

    let wish = create_wish_dir(&snapshot, forward, side);
    let max_accel = get_max_accel(&snapshot, &cv, wish, tick_interval);

    if player_vel - target_vel < max_accel {
        let scale = (player_vel - target_vel) / max_accel;
        forward *= scale;
        side *= scale;

        DECEL_ACTIVE.store(false, Ordering::Relaxed);
        process_cmd::unsubscribe(Subscriber::Decel(on_process_cmd));
    }

    schedule_movement(side, forward);
}

pub fn enable_decel(target_speed: f32) {
    DECEL_ACTIVE.store(true, Ordering::Relaxed);
    TARGET_SPEED.store(target_speed, Ordering::Relaxed);
    process_cmd::subscribe(Subscriber::Decel(on_process_cmd));
}

pub fn disable_decel() {
    if DECEL_ACTIVE.swap(false, Ordering::Relaxed) {
        process_cmd::unsubscribe(Subscriber::Decel(on_process_cmd));
    }
}

/// Stops as fast as possible via ground friction.
///
/// `target_speed:` horizontal speed threshold below which decel disables itself. By default set to 0
/// `enabled:` whether to enable decel tool. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_decel(target_speed: Option<f32>, enabled: Option<bool>) {
    let target_speed = target_speed.unwrap_or_default();
    let enabled = enabled.unwrap_or_else(|| {
        !DECEL_ACTIVE.load(Ordering::Relaxed)
            || target_speed != TARGET_SPEED.load(Ordering::Relaxed)
    });

    if enabled {
        enable_decel(target_speed);
        clog!(prefix;
            [Color::WHITE => "Decel "],
            [Color::LIME => "enabled"],
        );
    } else {
        disable_decel();
        clog!(prefix;
            [Color::WHITE => "Decel "],
            [Color::RED => "disabled"],
        );
    }
}
