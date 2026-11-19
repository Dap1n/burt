use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use crate::{
    clog,
    game::{
        entity::{self, edict_t},
        hooks::{
            Subscriber,
            create_move::{self, CreateMoveArgs},
            process_cmd,
        },
        math::{Vec3, calculate_aim_angles, shortest_dyaw},
        player,
        provider::provider,
    },
    structs::{Color, Coordinates, Easing, TargetEntity},
};
use burt_command::burt_command;
use portable_atomic::AtomicF32;

static AUTOAIM_ACTIVE: AtomicBool = AtomicBool::new(false);
static AIM_TARGET: [AtomicF32; 3] = [
    AtomicF32::new(0.0),
    AtomicF32::new(0.0),
    AtomicF32::new(0.0),
];
static START_PITCH: AtomicF32 = AtomicF32::new(0.0);
static START_YAW: AtomicF32 = AtomicF32::new(0.0);
static DURATION_COMPLETED: AtomicBool = AtomicBool::new(false);
static DURATION_TICKS: AtomicU32 = AtomicU32::new(0);
static ELAPSED_TICKS: AtomicU32 = AtomicU32::new(0);
static EASING: AtomicU8 = AtomicU8::new(0);

static LAST_VEL_X: AtomicF32 = AtomicF32::new(0.0);
static LAST_VEL_Y: AtomicF32 = AtomicF32::new(0.0);
static LAST_VEL_Z: AtomicF32 = AtomicF32::new(0.0);

static CURRENT_EDICT: AtomicU32 = AtomicU32::new(0);
static CURRENT_ENTITY: AtomicU32 = AtomicU32::new(0);

fn predict_eye() -> Option<Vec3> {
    let eye = player::eye_pos()?;
    let vel = player::velocity()?;

    let last_x = LAST_VEL_X.swap(vel.0, Ordering::Relaxed);
    let last_y = LAST_VEL_Y.swap(vel.1, Ordering::Relaxed);
    let last_z = LAST_VEL_Z.swap(vel.2, Ordering::Relaxed);

    let diff = |last: f32, curr: f32| {
        if last.abs() < 0.001 || curr.abs() < 0.001 {
            0.0
        } else {
            curr - last
        }
    };

    let tick_interval = 1.0 / provider().tickrate();
    let predicted_x = eye.0 + (vel.0 * tick_interval) + (0.5 * diff(last_x, vel.0) * tick_interval);
    let predicted_y = eye.1 + (vel.1 * tick_interval) + (0.5 * diff(last_y, vel.1) * tick_interval);
    let predicted_z = eye.2 + (vel.2 * tick_interval) + (0.5 * diff(last_z, vel.2) * tick_interval);

    Some((predicted_x, predicted_y, predicted_z))
}

fn on_create_move(_: &mut CreateMoveArgs) {
    if !AUTOAIM_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    let Some(predicted_eye) = predict_eye() else {
        return;
    };

    let edict = CURRENT_EDICT.load(Ordering::Relaxed);

    let target = if edict == 0 {
        (
            AIM_TARGET[0].load(Ordering::Relaxed),
            AIM_TARGET[1].load(Ordering::Relaxed),
            AIM_TARGET[2].load(Ordering::Relaxed),
        )
    } else {
        let entity = CURRENT_ENTITY.load(Ordering::Relaxed);
        if entity == 0
            || unsafe { entity::entity_from_edict(edict as *const edict_t) } != entity as *const u8
        {
            disable_autoaim();
            return;
        }
        let Some((x, y, z)) = (unsafe { entity::entity_origin(entity as *const u8) }) else {
            disable_autoaim();
            return;
        };
        (
            x + AIM_TARGET[0].load(Ordering::Relaxed),
            y + AIM_TARGET[1].load(Ordering::Relaxed),
            z + AIM_TARGET[2].load(Ordering::Relaxed),
        )
    };

    let (target_pitch, target_yaw) = calculate_aim_angles(predicted_eye, target);

    if DURATION_COMPLETED.load(Ordering::Relaxed) {
        create_move::schedule_angles(target_pitch, target_yaw);
        return;
    }

    let duration = DURATION_TICKS.load(Ordering::Relaxed);

    let elapsed = ELAPSED_TICKS.load(Ordering::Relaxed);
    let next_elapsed = elapsed.saturating_add(1);
    let progress = (next_elapsed as f32 / duration as f32).min(1.0);

    let easing = Easing::from_index(EASING.load(Ordering::Relaxed));
    let eased = easing.apply(progress);

    let start_pitch = START_PITCH.load(Ordering::Relaxed);
    let start_yaw = START_YAW.load(Ordering::Relaxed);

    let pitch = start_pitch + (target_pitch - start_pitch) * eased;
    let yaw = start_yaw + shortest_dyaw(target_yaw - start_yaw) * eased;

    create_move::schedule_angles(pitch, yaw);

    if next_elapsed >= duration {
        DURATION_COMPLETED.store(true, Ordering::Relaxed);
        ELAPSED_TICKS.store(0, Ordering::Relaxed);
    } else {
        ELAPSED_TICKS.store(next_elapsed, Ordering::Relaxed);
    }
}

#[inline]
fn reset_state() {
    AIM_TARGET[0].store(0.0, Ordering::Relaxed);
    AIM_TARGET[1].store(0.0, Ordering::Relaxed);
    AIM_TARGET[2].store(0.0, Ordering::Relaxed);
    START_PITCH.store(0.0, Ordering::Relaxed);
    START_YAW.store(0.0, Ordering::Relaxed);
    DURATION_COMPLETED.store(false, Ordering::Relaxed);
    DURATION_TICKS.store(0, Ordering::Relaxed);
    ELAPSED_TICKS.store(0, Ordering::Relaxed);
    EASING.store(0, Ordering::Relaxed);
    LAST_VEL_X.store(0.0, Ordering::Relaxed);
    LAST_VEL_Y.store(0.0, Ordering::Relaxed);
    LAST_VEL_Z.store(0.0, Ordering::Relaxed);
    CURRENT_EDICT.store(0, Ordering::Relaxed);
    CURRENT_ENTITY.store(0, Ordering::Relaxed);
}

pub fn enable_autoaim(coordinates: Coordinates, duration: u32, easing: Easing) {
    reset_state();

    AIM_TARGET[0].store(coordinates.x, Ordering::Relaxed);
    AIM_TARGET[1].store(coordinates.y, Ordering::Relaxed);
    AIM_TARGET[2].store(coordinates.z, Ordering::Relaxed);

    let (start_pitch, start_yaw) = process_cmd::cached_body_angles();
    START_PITCH.store(start_pitch, Ordering::Relaxed);
    START_YAW.store(start_yaw, Ordering::Relaxed);

    DURATION_COMPLETED.store(duration == 0, Ordering::Relaxed);
    DURATION_TICKS.store(duration, Ordering::Relaxed);
    EASING.store(easing.to_index(), Ordering::Relaxed);
    AUTOAIM_ACTIVE.store(true, Ordering::Relaxed);

    create_move::subscribe(Subscriber::Autoaim(on_create_move));
}

pub fn enable_autoaim_entity(
    edict: *mut edict_t,
    ent: *const u8,
    offset: Coordinates,
    duration: u32,
    easing: Easing,
) {
    reset_state();

    CURRENT_EDICT.store(edict as u32, Ordering::Relaxed);
    CURRENT_ENTITY.store(ent as u32, Ordering::Relaxed);

    AIM_TARGET[0].store(offset.x, Ordering::Relaxed);
    AIM_TARGET[1].store(offset.y, Ordering::Relaxed);
    AIM_TARGET[2].store(offset.z, Ordering::Relaxed);

    let (start_pitch, start_yaw) = process_cmd::cached_body_angles();
    START_PITCH.store(start_pitch, Ordering::Relaxed);
    START_YAW.store(start_yaw, Ordering::Relaxed);

    DURATION_COMPLETED.store(duration == 0, Ordering::Relaxed);
    DURATION_TICKS.store(duration, Ordering::Relaxed);
    EASING.store(easing.to_index(), Ordering::Relaxed);
    AUTOAIM_ACTIVE.store(true, Ordering::Relaxed);

    create_move::subscribe(Subscriber::Autoaim(on_create_move));
}

pub fn disable_autoaim() {
    if AUTOAIM_ACTIVE.swap(false, Ordering::Relaxed) {
        reset_state();
        create_move::unsubscribe(Subscriber::Autoaim(on_create_move));
    }
}

/// Automatically aims at certain coordinates.
///
/// `coordinates:` comma-separated coordinates in 3D space
/// `time:` duration in game ticks. If omitted or `0`, aiming is instant
/// `easing:` either `linear`, `sin`, `cubic` or `exp`. Default is `linear`
/// `enabled:` whether to enable autoaim. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_autoaim_point(
    coordinates: Coordinates,
    time: Option<u32>,
    easing: Option<Easing>,
    enabled: Option<bool>,
) {
    let duration = time.unwrap_or_default();
    let easing = easing.unwrap_or_default();

    let enabled = enabled.unwrap_or_else(|| {
        !AUTOAIM_ACTIVE.load(Ordering::Relaxed)
            || coordinates.x != AIM_TARGET[0].load(Ordering::Relaxed)
            || coordinates.y != AIM_TARGET[1].load(Ordering::Relaxed)
            || coordinates.z != AIM_TARGET[2].load(Ordering::Relaxed)
            || duration != DURATION_TICKS.load(Ordering::Relaxed)
            || easing.to_index() != EASING.load(Ordering::Relaxed)
    });

    if enabled {
        enable_autoaim(coordinates, duration, easing);
        clog!(prefix;
            [Color::WHITE => "Autoaim "],
            [Color::LIME => "enabled"],
        );
    } else {
        disable_autoaim();
        clog!(prefix;
            [Color::WHITE => "Autoaim "],
            [Color::RED => "disabled"],
        );
    }
}

/// Automatically aims at certain entity.
///
/// `entity:` target entity:
///   - By default, it searches by name or classname and returns the first match. Example: `radio`
///   - If an index in square brackets is given, it selects the n-th entity
///     with that name or classname (zero-based). Example: `box[1]`.
///     Note that `box[0]` is equivalent to `box`
///   - If a plain number is provided, it's treated as an entity index
/// `offset:` comma-separated x,y,z offsets from the entity origin in 3D space
/// `time:` duration in game ticks. If omitted or `0`, aiming is instant
/// `easing:` either `linear`, `sin`, `cubic` or `exp`. Default is `linear`
/// `enabled:` whether to enable autoaim. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_autoaim_entity(
    entity: TargetEntity,
    offset: Option<Coordinates>,
    time: Option<u32>,
    easing: Option<Easing>,
    enabled: Option<bool>,
) {
    let offset = offset.unwrap_or_default();
    let easing = easing.unwrap_or_default();

    let duration = time.unwrap_or_default();

    let Some((edict, ent)) = entity.fetch() else {
        clog!(err_prefix; "Can't find target entity");
        return;
    };

    let enabled = enabled.unwrap_or_else(|| {
        !AUTOAIM_ACTIVE.load(Ordering::Relaxed)
            || edict as u32 != CURRENT_EDICT.load(Ordering::Relaxed)
            || ent as u32 != CURRENT_ENTITY.load(Ordering::Relaxed)
            || offset.x != AIM_TARGET[0].load(Ordering::Relaxed)
            || offset.y != AIM_TARGET[1].load(Ordering::Relaxed)
            || offset.z != AIM_TARGET[2].load(Ordering::Relaxed)
            || duration != DURATION_TICKS.load(Ordering::Relaxed)
            || easing.to_index() != EASING.load(Ordering::Relaxed)
    });

    if enabled {
        enable_autoaim_entity(edict, ent, offset, duration, easing);
        clog!(prefix;
            [Color::WHITE => "Entity autoaim "],
            [Color::LIME => "enabled"],
        );
    } else {
        disable_autoaim();
        clog!(prefix;
            [Color::WHITE => "Entity autoaim "],
            [Color::RED => "disabled"],
        );
    }
}
