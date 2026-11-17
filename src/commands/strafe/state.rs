use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use super::math::normalize_signed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrafeType {
    Vectorial,
    VectorialCam,
    Angular,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Speed {
    Specified(f32),
    Current,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Current { use_vel_angle: bool },
    Specified(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub type_: StrafeType,
    pub dir: Direction,
    pub speed: Speed,
    pub no_pitch_lock: bool,
    pub let_speed_lock: bool,
}

impl Params {
    pub const fn new() -> Self {
        Self {
            type_: StrafeType::Vectorial,
            dir: Direction::Current {
                use_vel_angle: false,
            },
            speed: Speed::Max,
            no_pitch_lock: false,
            let_speed_lock: false,
        }
    }
}

pub fn parse_direction(s: Option<&str>) -> Option<Direction> {
    match s? {
        "forward" => Some(Direction::Current {
            use_vel_angle: false,
        }),
        "forwardvel" => Some(Direction::Current {
            use_vel_angle: true,
        }),
        "left" => Some(Direction::Specified(10000.0)),
        "right" => Some(Direction::Specified(-10000.0)),
        other => other
            .parse()
            .ok()
            .map(|deg| Direction::Specified(normalize_signed(deg))),
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static PARAMS: Mutex<Params> = Mutex::new(Params::new());

#[inline]
pub fn load_params() -> Params {
    *PARAMS.lock().unwrap()
}

#[inline]
pub fn store_params(params: Params) {
    *PARAMS.lock().unwrap() = params;
}

#[inline]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

#[inline]
pub fn store_enabled(value: bool) {
    ENABLED.store(value, Ordering::Relaxed);
}
