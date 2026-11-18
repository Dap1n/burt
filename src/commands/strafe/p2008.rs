use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use portable_atomic::AtomicF32;

use super::{
    PITCH_LOCK_LIMIT,
    math::*,
    state::{Direction, parse_direction},
};
use crate::{
    clog,
    commands::strafe::COS_EPS,
    define_bounds,
    game::{
        cinput::Action,
        hooks::{
            Subscriber,
            create_move::{self, CreateMoveArgs},
            process_cmd::{self, ProcessCmdArgs, schedule_movement},
        },
        math::{Vec3, deg_to_rad, rad_to_deg},
        player,
        provider::provider,
    },
    structs::Color,
};
use burt_command::burt_command;

const DUCKED_CAP_FACTOR: f32 = 0.1;
const STANDING_CAP_FACTOR: f32 = 0.5;

const MAXSPEED_CMP_EPS: f32 = 0.01;
const VEL_EPS: f32 = 1.0;
const YAW_EPS: f32 = 0.01;

#[inline]
pub fn speed_cap(cv: &CvarCache, ducked: bool) -> f32 {
    let factor = if ducked || (cv.maxspeed - 320.0).abs() < MAXSPEED_CMP_EPS {
        DUCKED_CAP_FACTOR
    } else {
        STANDING_CAP_FACTOR
    };
    cv.maxspeed * factor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Abh,
    Afh,
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub dir: Direction,
    pub mode: Mode,
    pub afh_length: f32,
}

impl Params {
    pub const fn new() -> Self {
        Self {
            dir: Direction::Current {
                use_vel_angle: false,
            },
            mode: Mode::Unspecified,
            afh_length: 1.0,
        }
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static PARAMS: Mutex<Params> = Mutex::new(Params::new());

static PLAN_SIDE: AtomicF32 = AtomicF32::new(0.0);
static PLAN_FORWARD: AtomicF32 = AtomicF32::new(0.0);

#[inline]
fn load_params() -> Params {
    *PARAMS.lock().unwrap()
}
#[inline]
fn store_params(p: Params) {
    *PARAMS.lock().unwrap() = p;
}
#[inline]
fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
#[inline]
fn store_enabled(v: bool) {
    ENABLED.store(v, Ordering::Relaxed);
}

#[inline]
fn jumped_current_frame() -> bool {
    player::is_grounded() && Action::Jump.is_held()
}

fn plan_tick(
    params: &Params,
    cv: &CvarCache,
    velocity: Vec3,
    ducked: bool,
    jumped: bool,
    cur_pitch: f32,
    cur_yaw: f32,
) -> (f32, f32) {
    let cap = speed_cap(cv, ducked);
    let speed = length2d(velocity);

    let Direction::Specified(target) = params.dir else {
        unreachable!();
    };

    let vel_angle_deg = if speed > VEL_EPS {
        rad_to_deg(velocity.1.atan2(velocity.0))
    } else {
        target
    };

    if jumped && speed >= cap {
        let hop_target = if target.abs() > 360.0 {
            vel_angle_deg
                + 90.0f32.copysign(target) * if params.mode == Mode::Abh { -1.0 } else { 1.0 }
        } else {
            target
        };
        let yaw = match params.mode {
            Mode::Abh => normalize_signed(hop_target + 180.0),
            Mode::Afh => normalize_signed(hop_target),
            Mode::Unspecified => unreachable!(),
        };
        let forward = match params.mode {
            Mode::Abh => 0.0,
            Mode::Afh => -params.afh_length,
            Mode::Unspecified => unreachable!(),
        };
        if normalize_signed(yaw - cur_yaw).abs() > YAW_EPS {
            create_move::schedule_angles(cur_pitch, yaw);
        }
        return (0.0, forward);
    }

    let will_be_grounded = predict_will_be_grounded(velocity);

    let snapshot = PlayerSnapshot {
        velocity,
        pitch: cur_pitch,
        yaw: cur_yaw,
        will_be_grounded,
        ducked,
        max_speed: cv.maxspeed,
        surface_friction: player::surface_friction(),
    };

    let target_speed = cv.maxvelocity * 2.0;
    let tick_interval = 1.0 / provider().tickrate();
    let turn_dir = turning_direction(&snapshot, cv, true, target_speed, target, tick_interval);
    let strafe_ang = get_strafe_angle(
        &snapshot,
        cv,
        target_speed,
        vel_angle_deg,
        target,
        turn_dir,
        tick_interval,
    );

    let (yaw, forward, side) = match params.mode {
        Mode::Abh => (
            normalize_signed(vel_angle_deg + 180.0),
            -strafe_ang.cos(),
            strafe_ang.sin(),
        ),
        Mode::Afh => (vel_angle_deg, strafe_ang.cos(), -strafe_ang.sin()),
        Mode::Unspecified => unreachable!(),
    };

    if normalize_signed(yaw - cur_yaw).abs() > YAW_EPS {
        create_move::schedule_angles(cur_pitch, yaw);
    }

    (side, forward)
}

fn on_create_move(_: &mut CreateMoveArgs) {
    if !enabled() {
        return;
    }
    let params = load_params();
    let Some(velocity) = player::velocity() else {
        return;
    };

    let cv = CvarCache::load();
    let ducked = player::is_ducked();
    let jumped = jumped_current_frame();
    let (cur_pitch, cur_yaw) = process_cmd::cached_body_angles();

    let (side, forward) = plan_tick(&params, &cv, velocity, ducked, jumped, cur_pitch, cur_yaw);

    PLAN_SIDE.store(side, Ordering::Relaxed);
    PLAN_FORWARD.store(forward, Ordering::Relaxed);
}

fn on_process_cmd(args: &mut ProcessCmdArgs) {
    if !enabled() || args.cmd.is_null() {
        return;
    }
    let Some(velocity) = player::velocity() else {
        return;
    };
    let (pitch, _yaw) = process_cmd::cached_body_angles();
    let will_be_grounded = predict_will_be_grounded(velocity);

    let (side, mut forward) = (
        PLAN_SIDE.load(Ordering::Relaxed),
        PLAN_FORWARD.load(Ordering::Relaxed),
    );

    if !will_be_grounded && pitch.abs() >= PITCH_LOCK_LIMIT {
        let c = deg_to_rad(pitch).cos();
        if c.abs() > COS_EPS {
            forward /= c;
        }
    }

    schedule_movement(side, forward);
}

fn freeze_targets(mut params: Params) -> Params {
    let mut target = None;
    let yaw = process_cmd::cached_body_angles().1;

    if let Direction::Current { use_vel_angle } = params.dir {
        let angle = if use_vel_angle {
            match player::velocity() {
                Some(v) if length2d(v) > VEL_EPS => {
                    let deg = rad_to_deg(v.1.atan2(v.0));
                    target = Some(deg);
                    deg
                }
                _ => yaw,
            }
        } else {
            yaw
        };
        params.dir = Direction::Specified(angle);
    }

    if params.mode == Mode::Unspecified {
        params.mode = Mode::Afh;
        let Some(target) = target else {
            return params;
        };
        if normalize_signed(target - yaw).abs() > 90.0 {
            params.mode = Mode::Abh;
        }
    }

    params
}

pub fn enable_strafe(params: Params) {
    store_params(freeze_targets(params));
    if enabled() {
        return;
    }

    PLAN_SIDE.store(0.0, Ordering::Relaxed);
    PLAN_FORWARD.store(0.0, Ordering::Relaxed);

    store_enabled(true);
    create_move::subscribe(Subscriber::Strafe(on_create_move));
    process_cmd::subscribe(Subscriber::Strafe(on_process_cmd));
}

pub fn disable_strafe() {
    if !enabled() {
        return;
    }
    store_enabled(false);
    PLAN_SIDE.store(0.0, Ordering::Relaxed);
    PLAN_FORWARD.store(0.0, Ordering::Relaxed);
    create_move::unsubscribe(Subscriber::Strafe(on_create_move));
    process_cmd::unsubscribe(Subscriber::Strafe(on_process_cmd));
}

/// TAS strafe tool in Project 2008.
/// There's no `enabled` parameter. To disable autostrafer, write `burt_strafe off` or `burt_strafe none`.
///
/// `direction:` strafe direction control:
///   - `forward` — try to strafe in a straight line, towards the current view angle (default)
///   - `forwardvel` — try to strafe in a straight line, towards the current velocity angle
///   - `left` — try to strafe left
///   - `right` — try to strafe right
///   - if float `(f32)` is provided, sets a target yaw angle by the provided amount of degrees
/// `afh:` whether to accelerate forward:
///   - `false` — use Accelerated Back Hopping (ABH)
///   - `true` — use Accelerated Forward Hopping (AFH)
///   - if not provided, guesses the intention automatically by looking at
///     current angle and requested direction (or current velocity)
/// `afh_length:` number in range [0.0, 1.0] setting the magnitude of the backward input
///   injected on AFH hop ticks. 1.0 = full force (default)
#[burt_command(cheat)]
pub fn burt_strafe(direction: Option<String>, afh: Option<bool>, afh_length: Option<f32>) {
    let mut params = Params::new();

    if let Some(d) = direction.map(|s| s.to_lowercase()) {
        if ["off", "none"].contains(&d.as_str()) {
            disable_strafe();
            clog!(prefix;
                [Color::WHITE => "Strafing "],
                [Color::RED => "disabled"],
            );
            return;
        }
        if let Some(dir) = parse_direction(Some(&d)) {
            params.dir = dir;
        } else {
            clog!(err_prefix;
                [Color::WHITE => "Unknown direction "],
                [Color::RED => "{d}"],
            );
            return;
        }
    }

    match afh {
        Some(true) => params.mode = Mode::Afh,
        Some(false) => params.mode = Mode::Abh,
        None => params.mode = Mode::Unspecified,
    }

    if let Some(afh_length) = afh_length {
        define_bounds!(0.0 <= afh_length <= 1.0);
        params.afh_length = afh_length;
    }

    enable_strafe(params);
    clog!(prefix;
        [Color::WHITE => "Strafing "],
        [Color::LIME => "enabled"],
    );
}
