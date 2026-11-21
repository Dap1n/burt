use std::{
    mem,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};

use detour::static_detour;

use crate::{
    commands::{
        bvars::BVAR_TAS_SKIP_TO,
        tas::{SESSION_COUNTER, TICK},
    },
    game::signatures::{self, get_accumulator_addr, get_filtered_frametime_addr},
};

pub static TIME_PAUSED: AtomicBool = AtomicBool::new(false);
pub static TIME_ADVANCE_TICKS: AtomicU32 = AtomicU32::new(0);

fn dispatch(frametime: f32) {
    unsafe {
        HOOK.call(frametime);
    }

    let in_tas = SESSION_COUNTER.load(Ordering::Relaxed) == 1;
    let advance = TIME_ADVANCE_TICKS.load(Ordering::Relaxed);

    if in_tas && advance == 0 {
        let accumulator = get_accumulator_addr() as *mut f32;
        unsafe {
            *accumulator = 0.0;
        }
    }

    let skip_to = BVAR_TAS_SKIP_TO.load(Ordering::Relaxed);
    let skipping = in_tas && skip_to != 0 && TICK.load(Ordering::Relaxed) <= skip_to;

    if skipping || !TIME_PAUSED.load(Ordering::Relaxed) {
        return;
    }

    if advance != 0 {
        TIME_ADVANCE_TICKS.fetch_sub(1, Ordering::Relaxed);
        return;
    }

    if !in_tas {
        let accumulator = get_accumulator_addr() as *mut f32;
        unsafe {
            *accumulator = 0.0;
        }
    }

    let filtered_frametime = get_filtered_frametime_addr() as *mut f32;
    unsafe {
        *filtered_frametime = 0.0;
    }
}

type HostFilterTimeFn = unsafe extern "cdecl" fn(f32);

static_detour! {
    static HOOK: unsafe extern "cdecl" fn(f32);
}

pub(super) unsafe fn install() -> Result<(), detour::Error> {
    unsafe {
        let target: HostFilterTimeFn = mem::transmute(signatures::get_host_filter_time_addr());
        HOOK.initialize(target, dispatch)?;
        HOOK.enable()?;
    }
    Ok(())
}

pub(super) unsafe fn uninstall() -> Result<(), detour::Error> {
    unsafe {
        HOOK.disable()?;
    }
    Ok(())
}
