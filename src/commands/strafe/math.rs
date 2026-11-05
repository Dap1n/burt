use super::state::Params;
use crate::{
    commands::strafe::state::{Direction, Speed},
    game::{
        cinput::Action,
        cvars::*,
        math::{Vec3, deg_to_rad, rad_to_deg},
        player::{self, TICKTIME},
    },
};

const AIRACCELERATE: f32 = 10.0;
const AIRCONTROL_LIMIT: f32 = 300.0;

#[inline]
pub fn normalize_signed(deg: f32) -> f32 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

#[inline]
pub fn length2d(v: Vec3) -> f32 {
    v.0.hypot(v.1)
}

#[inline]
pub fn length3d(v: Vec3) -> f32 {
    (v.0.powi(2) + v.1.powi(2) + v.2.powi(2)).sqrt()
}

#[derive(Clone, Copy, Debug)]
pub struct CvarCache {
    pub friction: f32,
    pub stopspeed: f32,
    pub accelerate: f32,
    pub maxspeed: f32,
    pub maxvelocity: f32,
}

impl CvarCache {
    pub fn load() -> Self {
        Self {
            friction: SV_FRICTION.float_or(4.0),
            stopspeed: SV_STOPSPEED.float_or(100.0),
            accelerate: SV_ACCELERATE.float_or(10.0),
            maxspeed: SV_MAXSPEED.float_or(320.0),
            maxvelocity: SV_MAXVELOCITY.float_or(3500.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerSnapshot {
    pub velocity: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub will_be_grounded: bool,
    pub ducked: bool,
    pub max_speed: f32,
    pub surface_friction: f32,
}

#[inline]
fn total_friction(p: &PlayerSnapshot, cv: &CvarCache) -> f32 {
    cv.friction * p.surface_friction
}

pub fn ground_friction_velocity(p: &PlayerSnapshot, cv: &CvarCache) -> Vec3 {
    if !p.will_be_grounded {
        return p.velocity;
    }

    let mut vel = p.velocity;
    let speed = length2d(vel);
    let friction = total_friction(p, cv);

    if speed >= cv.stopspeed {
        let scale = 1.0 - TICKTIME * friction;
        vel.0 *= scale;
        vel.1 *= scale;
    } else {
        let min_speed = 0.1f32.max(TICKTIME * cv.stopspeed * friction);
        if speed >= min_speed {
            let inv_speed = if speed > 0.0 { 1.0 / speed } else { 0.0 };
            let nx = vel.0 * inv_speed;
            let ny = vel.1 * inv_speed;
            let drop = TICKTIME * cv.stopspeed * friction;
            vel.0 -= nx * drop;
            vel.1 -= ny * drop;
        } else {
            vel.0 = 0.0;
            vel.1 = 0.0;
        }
    }

    if length2d(vel) < 1.0 {
        vel.0 = 0.0;
        vel.1 = 0.0;
    }

    vel
}

pub fn get_max_speed(p: &PlayerSnapshot, wish: (f32, f32), not_aired: bool) -> f32 {
    let duck_multiplier = if p.will_be_grounded && p.ducked {
        1.0 / 3.0
    } else {
        1.0
    };

    let wx = wish.0 * p.max_speed;
    let wy = wish.1 * p.max_speed;
    let max_speed = p.max_speed.min(wx.hypot(wy)) * duck_multiplier;

    if p.will_be_grounded || not_aired {
        max_speed
    } else {
        max_speed.min(30.0)
    }
}

pub fn get_max_accel(p: &PlayerSnapshot, cv: &CvarCache, wish: (f32, f32)) -> f32 {
    let accel = if p.will_be_grounded {
        cv.accelerate
    } else {
        AIRACCELERATE
    };

    p.surface_friction * TICKTIME * get_max_speed(p, wish, p.will_be_grounded) * accel
}

pub fn predict_will_be_grounded(velocity: Vec3) -> bool {
    player::is_grounded() && !Action::Jump.is_held() && velocity.2 <= 0.0
}

pub fn air_lock_factor(p: &PlayerSnapshot) -> (f32, f32) {
    if p.will_be_grounded {
        return (0.0, 0.0);
    }

    let speed = length2d(p.velocity);
    if speed <= AIRCONTROL_LIMIT {
        return (0.0, 0.0);
    }

    let mut factor = (0.0, 0.0);

    let abs_x = p.velocity.0.abs();
    if abs_x > AIRCONTROL_LIMIT * 0.5 {
        factor.0 = p.velocity.0 / abs_x;
    }

    let abs_y = p.velocity.1.abs();
    if abs_y > AIRCONTROL_LIMIT * 0.5 {
        factor.1 = p.velocity.1 / abs_y;
    }

    factor
}

pub fn create_wish_dir(p: &PlayerSnapshot, forward_move: f32, side_move: f32) -> (f32, f32) {
    let mut wish = (side_move, forward_move);

    if !p.will_be_grounded && p.pitch.abs() >= 30.0 {
        wish.1 *= deg_to_rad(p.pitch).cos();
    }

    let yaw = deg_to_rad(p.yaw);
    let sin = yaw.sin();
    let cos = yaw.cos();

    let mut world = (sin * wish.0 + cos * wish.1, -cos * wish.0 + sin * wish.1);

    let lock = air_lock_factor(p);
    if lock.0 * world.0 < 0.0 {
        world.0 = 0.0;
    }
    if lock.1 * world.1 < 0.0 {
        world.1 = 0.0;
    }

    world
}

pub fn get_velocity_after_move(
    p: &PlayerSnapshot,
    cv: &CvarCache,
    forward_move: f32,
    side_move: f32,
    will_be_grounded: bool,
) -> Vec3 {
    let mut velocity = ground_friction_velocity(p, cv);
    if will_be_grounded {
        velocity.2 -= 10.0;
    }

    let wish = create_wish_dir(p, forward_move, side_move);
    let wishspeed = (wish.0 * wish.0 + wish.1 * wish.1).sqrt();

    if wishspeed == 0.0 {
        return velocity;
    }

    let nx = wish.0 / wishspeed;
    let ny = wish.1 / wishspeed;

    let accel = if p.will_be_grounded {
        cv.accelerate
    } else {
        AIRACCELERATE
    };

    let clamped_wishspeed = if p.will_be_grounded {
        wishspeed.min(p.max_speed)
    } else {
        wishspeed.min(30.0)
    };
    let dot = velocity.0 * nx + velocity.1 * ny;
    let addspeed = clamped_wishspeed - dot;

    if addspeed <= 0.0 {
        velocity
    } else {
        let accelspeed = p.surface_friction * TICKTIME * wishspeed * accel;
        let accel_force = accelspeed.min(addspeed);

        (
            velocity.0 + nx * accel_force,
            velocity.1 + ny * accel_force,
            velocity.2,
        )
    }
}

pub fn get_fastest_strafe_angle(p: &PlayerSnapshot, cv: &CvarCache) -> f32 {
    let velocity = ground_friction_velocity(p, cv);
    let speed = length2d(velocity);
    if speed == 0.0 {
        return 0.0;
    }

    let wish = (0.0, 1.0);
    let max_speed = get_max_speed(p, wish, p.will_be_grounded);
    let max_accel = get_max_accel(p, cv, wish);

    let cos_ang = (max_speed - max_accel) / speed;
    cos_ang.clamp(-1.0, 1.0).acos()
}

pub fn get_turning_strafe_angle(p: &PlayerSnapshot, cv: &CvarCache) -> f32 {
    let velocity = ground_friction_velocity(p, cv);
    let speed = length2d(velocity);
    if speed == 0.0 {
        return 0.0;
    }

    let max_accel = get_max_accel(p, cv, (0.0, 1.0));

    let mut cos_ang = -max_accel / speed;
    if cos_ang < -1.0 {
        cos_ang = 0.0;
    }

    cos_ang.clamp(-1.0, 1.0).acos()
}

pub fn get_target_strafe_angle(
    p: &PlayerSnapshot,
    cv: &CvarCache,
    target_speed: f32,
    turning_dir: i32,
) -> f32 {
    let velocity = ground_friction_velocity(p, cv);
    let current_speed = length2d(velocity);
    if current_speed == 0.0 {
        return 0.0;
    }

    let max_accel = get_max_accel(p, cv, (0.0, 1.0));

    let denom = 2.0 * current_speed * max_accel;
    if denom.abs() < f32::EPSILON {
        return 0.0;
    }

    let arg = -(current_speed * current_speed + max_accel * max_accel
        - target_speed * target_speed)
        / denom;

    arg.clamp(-1.0, 1.0).acos() * turning_dir as f32
}

pub fn turning_direction(
    p: &PlayerSnapshot,
    cv: &CvarCache,
    params: &Params,
    target_speed: f32,
    target_angle_deg: f32,
) -> i32 {
    let speed = length2d(p.velocity);
    let vel_angle_deg = if speed == 0.0 {
        target_angle_deg
    } else {
        rad_to_deg(p.velocity.1.atan2(p.velocity.0))
    };
    let diff = normalize_signed(target_angle_deg - vel_angle_deg);

    let mut turn_dir = if target_angle_deg < -180.0 || (target_angle_deg <= 180.0 && diff < 0.0) {
        -1
    } else {
        1
    };

    if !params.let_speed_lock
        && !p.will_be_grounded
        && speed < target_speed
        && speed >= AIRCONTROL_LIMIT
    {
        let wish = (0.0, 1.0);
        let max_speed = get_max_speed(p, wish, p.will_be_grounded);
        let max_accel = get_max_accel(p, cv, wish);
        let speed_capped_vel = max_speed - max_accel;

        if p.velocity.0.abs() > speed_capped_vel && p.velocity.1.abs() > speed_capped_vel {
            let cardinal = -(deg_to_rad(vel_angle_deg) * 4.0).sin();
            if cardinal > 0.0 {
                turn_dir = 1;
            } else if cardinal < 0.0 {
                turn_dir = -1;
            }
        }
    }

    turn_dir
}

pub fn resolve_target_speed(p: &PlayerSnapshot, cv: &CvarCache, params: &Params) -> f32 {
    match params.speed {
        Speed::Specified(value) => value,
        Speed::Current => length2d(p.velocity),
        Speed::Max => cv.maxvelocity * 2.0,
    }
}

pub fn resolve_target_angle(params: &Params, fallback_yaw: f32) -> f32 {
    match params.dir {
        Direction::Specified(angle) => angle,
        Direction::Current { .. } => fallback_yaw,
    }
}

pub fn get_strafe_angle(
    p: &PlayerSnapshot,
    cv: &CvarCache,
    target_speed: f32,
    vel_angle_deg: f32,
    target_angle_deg: f32,
    turning_dir: i32,
) -> f32 {
    let velocity = ground_friction_velocity(p, cv);
    let speed = length2d(velocity);

    let mut speed_diff = target_speed - speed;
    if speed_diff.abs() < 0.001 {
        speed_diff = 0.0;
    }

    let td = turning_dir as f32;
    if speed_diff > 0.0 {
        let mut a = get_fastest_strafe_angle(p, cv) * td;

        let vel_angle_rad = if speed == 0.0 {
            deg_to_rad(target_angle_deg)
        } else {
            deg_to_rad(vel_angle_deg)
        };
        let correct_ang = vel_angle_rad + a - deg_to_rad(p.yaw);
        let forward = correct_ang.cos();
        let side = -correct_ang.sin();
        let predicted_speed = length2d(get_velocity_after_move(p, cv, forward, side, false));
        if predicted_speed > target_speed {
            a = get_target_strafe_angle(p, cv, target_speed, turning_dir);
        }
        a
    } else if speed_diff < 0.0 {
        get_target_strafe_angle(p, cv, target_speed, turning_dir)
    } else {
        get_turning_strafe_angle(p, cv) * td
    }
}
