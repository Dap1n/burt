use std::sync::atomic::Ordering;

use super::math::*;
use crate::{
    clog,
    commands::{
        strafe::{
            COS_EPS, PITCH_LOCK_LIMIT,
            state::{self, *},
        },
        tas::Point2D,
    },
    game::{
        hooks::{
            Subscriber,
            create_move::{self, CreateMoveArgs},
            process_cmd::{self, ProcessCmdArgs, SCHEDULED_MOVEMENT, schedule_movement},
        },
        math::{deg_to_rad, rad_to_deg},
        player,
    },
    structs::Color,
};
use burt_command::burt_command;
use portable_atomic::AtomicF32;

const VEL_EPS: f32 = 1.0;

static ABS_MOVE_ANGLE: AtomicF32 = AtomicF32::new(0.0);
static HELD_REL: AtomicF32 = AtomicF32::new(0.0);

fn on_create_move(_: &mut CreateMoveArgs) {
    if !state::enabled() {
        return;
    }
    let params = load_params();
    let Some(velocity) = player::velocity() else {
        return;
    };
    let will_be_grounded = predict_will_be_grounded(velocity);
    let ducked = player::is_ducked();
    let (cur_pitch, cur_yaw) = process_cmd::cached_body_angles();
    let cv = CvarCache::load();

    let target_angle_deg = resolve_target_angle(&params, cur_yaw);

    let speed = length2d(velocity);
    let vel_angle_deg = if speed > VEL_EPS {
        rad_to_deg(velocity.1.atan2(velocity.0))
    } else {
        target_angle_deg
    };

    let snapshot = PlayerSnapshot {
        velocity,
        pitch: cur_pitch,
        yaw: cur_yaw,
        will_be_grounded,
        ducked,
        max_speed: cv.maxspeed,
        surface_friction: player::surface_friction(),
    };
    let target_speed = resolve_target_speed(&snapshot, &cv, &params);
    let turn_dir = turning_direction(
        &snapshot,
        &cv,
        params.let_speed_lock,
        target_speed,
        target_angle_deg,
    );
    let strafe_ang = get_strafe_angle(
        &snapshot,
        &cv,
        target_speed,
        vel_angle_deg,
        target_angle_deg,
        turn_dir,
    );
    let abs_move_angle_deg = vel_angle_deg + rad_to_deg(strafe_ang);
    ABS_MOVE_ANGLE.store(abs_move_angle_deg, Ordering::Relaxed);

    let target_pitch =
        if !will_be_grounded && cur_pitch.abs() >= PITCH_LOCK_LIMIT && !params.no_pitch_lock {
            29.9999f32.copysign(cur_pitch)
        } else {
            cur_pitch
        };

    let target_yaw = match params.type_ {
        StrafeType::Vectorial => cur_yaw,
        StrafeType::VectorialCam => {
            if speed > VEL_EPS {
                vel_angle_deg
            } else {
                cur_yaw
            }
        }
        StrafeType::Angular => abs_move_angle_deg - HELD_REL.load(Ordering::Relaxed),
    };

    if target_pitch != cur_pitch || target_yaw != cur_yaw {
        create_move::schedule_angles(target_pitch, target_yaw);
    }
}

fn on_process_cmd(args: &mut ProcessCmdArgs) {
    if !state::enabled() || args.cmd.is_null() {
        return;
    }
    let params = load_params();
    let Some(velocity) = player::velocity() else {
        return;
    };

    let (pitch, yaw) = process_cmd::cached_body_angles();
    let will_be_grounded = predict_will_be_grounded(velocity);

    let abs_move_angle_deg = ABS_MOVE_ANGLE.load(Ordering::Relaxed);

    let (mut forward, side) = match params.type_ {
        StrafeType::Vectorial | StrafeType::VectorialCam => {
            let rel = deg_to_rad(abs_move_angle_deg - yaw);
            (rel.cos(), -rel.sin())
        }
        StrafeType::Angular => {
            let Point2D { x, y } = Point2D::from_bits(SCHEDULED_MOVEMENT.load(Ordering::Relaxed));
            let held_rel = (-x).atan2(y);
            HELD_REL.store(held_rel.to_degrees(), Ordering::Relaxed);
            (held_rel.cos(), -held_rel.sin())
        }
    };

    if !will_be_grounded && pitch.abs() >= PITCH_LOCK_LIMIT && params.no_pitch_lock {
        let c = deg_to_rad(pitch).cos();
        if c.abs() > COS_EPS {
            forward /= c;
        }
    }

    schedule_movement(side, forward);
}

fn freeze_targets(mut params: Params) -> Params {
    if let Direction::Current { use_vel_angle } = params.dir {
        let yaw = process_cmd::cached_body_angles().1;
        let angle = if use_vel_angle {
            match player::velocity() {
                Some(v) if length2d(v) > VEL_EPS => rad_to_deg(v.1.atan2(v.0)),
                _ => yaw,
            }
        } else {
            yaw
        };
        params.dir = Direction::Specified(angle);
    }

    if params.speed == Speed::Current {
        params.speed = Speed::Specified(player::velocity().map(length2d).unwrap_or_default());
    }

    params
}

pub fn enable_strafe(params: Params) {
    store_params(freeze_targets(params));
    HELD_REL.store(0.0, Ordering::Relaxed);
    store_enabled(true);
    create_move::subscribe(Subscriber::Strafe(on_create_move));
    process_cmd::subscribe(Subscriber::Strafe(on_process_cmd));
}

pub fn disable_strafe() {
    if !state::enabled() {
        return;
    }
    store_params(Params::new());
    store_enabled(false);
    create_move::unsubscribe(Subscriber::Strafe(on_create_move));
    process_cmd::unsubscribe(Subscriber::Strafe(on_process_cmd));
}

/// TAS strafe tool in July 2009 build.
/// There's no `enabled` parameter. To disable autostrafer, write `burt_strafe off` or `burt_strafe none`.
///
/// `mode:` strafing type control:
///   - `vec` — enables vectorial strafing (movement analog is adjusted to get desired movement direction) (default)
///   - `ang` — enables angular strafing (view analog is adjusted to get desired movement direction)
///   - `veccam` — enables special vectorial strafing that rotates you towards your current moving direction
///
/// `velocity:` velocity control:
///   - `max` — aim for the greatest acceleration (default)
///   - `keep` — maintain the current velocity
///   - if float `(f32)` is provided, sets a target velocity by the provided units per second
///
/// `direction:` strafe direction control:
///   - `forward` — try to strafe in a straight line, towards the current view angle (default)
///   - `forwardvel` — try to strafe in a straight line, towards the current velocity angle
///   - `left` — try to strafe left
///   - `right` — try to strafe right
///   - if float `(f32)` is provided, sets a target yaw angle by the provided amount of degrees
///
/// `no_pitch_lock:` make the autostrafer not clamp the pitch. The autostrafer will always
/// clamp your pitch angle (up and down) between -30 and 30 when midair, as it gives the fastest
/// possible acceleration. This argument will tell the autostrafer that you wish to enable sub-optimal
/// strafing. By default set to `false`
///
/// `let_speed_lock:` by default set to `false`, meaning autostrafer will attempt to not deviate
/// too much right or left when the velocity is higher than or equal 300 ups, to avoid speedlock
/// and keep acceleration at its maximum. Sometimes that behaviour is not wanted, in which case
/// this parameter should be used
#[burt_command(cheat)]
pub fn burt_strafe(
    mode: Option<String>,
    velocity: Option<String>,
    direction: Option<String>,
    no_pitch_lock: Option<bool>,
    let_speed_lock: Option<bool>,
) {
    let mut params = Params::new();

    match mode.map(|s| s.to_lowercase()).as_deref() {
        Some("off" | "none") => {
            disable_strafe();
            clog!(prefix;
                [Color::WHITE => "Strafe "],
                [Color::RED => "disabled"],
            );
            return;
        }
        Some("vec" | "vectorial") | None => params.type_ = StrafeType::Vectorial,
        Some("ang" | "angular") => params.type_ = StrafeType::Angular,
        Some("veccam" | "vectorialcam") => params.type_ = StrafeType::VectorialCam,
        Some(mode) => {
            clog!(err_prefix;
                [Color::WHITE => "Unknown mode "],
                [Color::RED => "{}", mode],
            );
            return;
        }
    }

    match velocity.map(|s| s.to_lowercase()).as_deref() {
        Some("max") | None => params.speed = Speed::Max,
        Some("keep") => params.speed = Speed::Current,
        Some(ups_str) if let Ok(ups) = ups_str.parse() => {
            if ups < 0.0 {
                clog!(err_prefix;
                    [Color::WHITE => "Parameter "],
                    [Color::RED => "velocity"],
                    [Color::WHITE => " expects a non-negative value"],
                );
                return;
            }
            params.speed = Speed::Specified(ups);
        }
        Some(vel) => {
            clog!(err_prefix;
                [Color::WHITE => "Unknown velocity "],
                [Color::RED => "{}", vel],
            );
            return;
        }
    }

    match direction.map(|s| s.to_lowercase()).as_deref() {
        Some("forward") | None => {
            params.dir = Direction::Current {
                use_vel_angle: false,
            };
        }
        Some("forwardvel") => {
            params.dir = Direction::Current {
                use_vel_angle: true,
            };
        }
        Some("left") => {
            params.dir = Direction::Specified(10000.0);
        }
        Some("right") => {
            params.dir = Direction::Specified(-10000.0);
        }
        Some(deg_str) if let Ok(deg) = deg_str.parse() => {
            params.dir = Direction::Specified(normalize_signed(deg));
        }
        Some(dir) => {
            clog!(err_prefix;
                [Color::WHITE => "Unknown direction "],
                [Color::RED => "{}", dir],
            );
            return;
        }
    }

    if no_pitch_lock.unwrap_or_default() {
        params.no_pitch_lock = true;
    }
    if let_speed_lock.unwrap_or_default() {
        params.let_speed_lock = true;
    }

    enable_strafe(params);
    clog!(prefix;
        [Color::WHITE => "Strafe "],
        [Color::LIME => "enabled"],
    );
}
