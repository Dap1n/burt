use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    clog,
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

static AUTOJUMP_ACTIVE: AtomicBool = AtomicBool::new(false);
static DUCKED: AtomicBool = AtomicBool::new(false);
static JUMP_HELD: AtomicBool = AtomicBool::new(false);
static DUCK_HELD: AtomicBool = AtomicBool::new(false);

fn on_create_move(_args: &mut CreateMoveArgs) {
    if !AUTOJUMP_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    if JUMP_HELD.swap(false, Ordering::Relaxed) {
        if DUCKED.load(Ordering::Relaxed) {
            DUCK_HELD.store(true, Ordering::Relaxed);
        }
        Action::Jump.release(0);
    } else if player::is_grounded() {
        if DUCKED.load(Ordering::Relaxed) && !DUCK_HELD.load(Ordering::Relaxed) {
            Action::Duck.press(0);
        }
        Action::Jump.press(0);
        JUMP_HELD.store(true, Ordering::Relaxed);
    } else if DUCK_HELD.swap(false, Ordering::Relaxed) {
        Action::Duck.release(0);
    }
}

pub fn enable_autojump(ducked: bool) {
    AUTOJUMP_ACTIVE.store(true, Ordering::Relaxed);
    DUCKED.store(ducked, Ordering::Relaxed);
    create_move::subscribe(Subscriber::Autojump(on_create_move));
}

pub fn disable_autojump() {
    if !AUTOJUMP_ACTIVE.swap(false, Ordering::Relaxed) {
        return;
    }
    create_move::unsubscribe(Subscriber::Autojump(on_create_move));
    if JUMP_HELD.swap(false, Ordering::Relaxed) {
        Action::Jump.release(0);
    }
    if DUCK_HELD.swap(false, Ordering::Relaxed) {
        Action::Duck.release(0);
    }
}

/// Toggles automatic jumping when getting grounded.
///
/// `ducked:` enables autojump while also ducking. Ducking slightly increases your jump height. By default set to `false`
/// `enabled:` whether to enable autojump. By default guesses the intention automatically
#[burt_command(cheat)]
pub fn burt_autojump(ducked: Option<bool>, enabled: Option<bool>) {
    let ducked = ducked.unwrap_or_default();
    let enabled = enabled.unwrap_or_else(|| {
        !AUTOJUMP_ACTIVE.load(Ordering::Relaxed) || ducked != DUCKED.load(Ordering::Relaxed)
    });
    if enabled {
        enable_autojump(ducked);
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
