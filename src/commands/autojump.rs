use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::{
    clog,
    engine_build::{self, EngineBuild},
    game::{
        cinput::Action,
        hooks::{
            Subscriber,
            create_move::{self, CreateMoveArgs},
        },
        player,
    },
    structs::Color,
};
use burt_command::burt_command;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DuckState {
    #[default]
    Disabled = 0,
    Partial = 1,
    Full = 2,
}

impl DuckState {
    #[inline]
    const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Disabled,
            1 => Self::Partial,
            2 => Self::Full,
            _ => unreachable!(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ducked" => Self::Partial,
            "?ducked" => Self::Full,
            _ => Self::Disabled,
        }
    }
}

static AUTOJUMP_ACTIVE: AtomicBool = AtomicBool::new(false);
static DUCK_STATE: AtomicU8 = AtomicU8::new(0);
static JUMP_HELD: AtomicBool = AtomicBool::new(false);
static DUCK_HELD: AtomicBool = AtomicBool::new(false);

fn on_create_move(_: &mut CreateMoveArgs) {
    if !AUTOJUMP_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let duck_state = DuckState::from_index(DUCK_STATE.load(Ordering::Relaxed));
    if duck_state == DuckState::Full && !DUCK_HELD.load(Ordering::Relaxed) {
        DUCK_HELD.store(true, Ordering::Relaxed);
        Action::Duck.press(0);
    }
    if JUMP_HELD.swap(false, Ordering::Relaxed) {
        Action::Jump.release(0);
    } else if player::is_grounded() {
        if duck_state == DuckState::Partial && !DUCK_HELD.load(Ordering::Relaxed) {
            DUCK_HELD.store(true, Ordering::Relaxed);
            Action::Duck.press(0);
        }
        Action::Jump.press(0);
        JUMP_HELD.store(true, Ordering::Relaxed);
    } else if duck_state == DuckState::Partial && DUCK_HELD.swap(false, Ordering::Relaxed) {
        Action::Duck.release(0);
    }
}

fn restore() {
    if JUMP_HELD.swap(false, Ordering::Relaxed) {
        Action::Jump.release(0);
    }
    if DUCK_HELD.swap(false, Ordering::Relaxed) {
        Action::Duck.release(0);
    }
}

pub fn enable_autojump(duck_state: DuckState) {
    AUTOJUMP_ACTIVE.store(true, Ordering::Relaxed);
    DUCK_STATE.store(duck_state as u8, Ordering::Relaxed);
    restore();
    create_move::subscribe(Subscriber::Autojump(on_create_move));
}

pub fn disable_autojump() {
    if !AUTOJUMP_ACTIVE.swap(false, Ordering::Relaxed) {
        return;
    }
    restore();
    create_move::unsubscribe(Subscriber::Autojump(on_create_move));
}

/// Toggles automatic jumping when getting grounded.
///
/// `duck_state:` enables autojump while also ducking
///   - If set to `ducked`, will only duck when touching the ground
///   - If set to `?ducked`, ducking will be constant (only available in Project 2008)
/// `enabled:` whether to enable autojump. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_autojump(duck_state: Option<DuckState>, enabled: Option<bool>) {
    let duck_state = duck_state.unwrap_or_default();
    if duck_state == DuckState::Full && engine_build::current_build() != EngineBuild::Project2008 {
        clog!(err_prefix; "Autojump with constant ducking can only be used in Project 2008");
        return;
    }
    let enabled = enabled.unwrap_or_else(|| {
        !AUTOJUMP_ACTIVE.load(Ordering::Relaxed)
            || duck_state as u8 != DUCK_STATE.load(Ordering::Relaxed)
    });
    if enabled {
        enable_autojump(duck_state);
        clog!(prefix;
            [Color::WHITE => "Autojump "],
            [Color::LIME => "enabled"],
        );
    } else {
        disable_autojump();
        clog!(prefix;
            [Color::WHITE => "Autojump "],
            [Color::RED => "disabled"],
        );
    }
}
