use std::{
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use crate::{
    commands::{
        bvars::BVAR_TAS_SKIP_TO,
        tas::{SESSION_COUNTER, TICK},
    },
    game::signatures::{self, get_accumulator_addr, get_filtered_frametime_addr},
    log,
    os::*,
};

pub type HostFilterTimeFn = unsafe extern "cdecl" fn(frametime: f32);

pub static TIME_PAUSED: AtomicBool = AtomicBool::new(false);
pub static TIME_ADVANCE_TICKS: AtomicU32 = AtomicU32::new(0);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGET: OnceLock<usize> = OnceLock::new();
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static ORIG_FN: OnceLock<HostFilterTimeFn> = OnceLock::new();

unsafe extern "cdecl" fn dispatch(frametime: f32) {
    let (Some(&target), Some(bytes), Some(&orig)) = (TARGET.get(), ORIG_BYTES.get(), ORIG_FN.get())
    else {
        return;
    };

    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        orig(frametime);

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
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

pub(super) unsafe fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let target = signatures::get_host_filter_time_addr();
    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);
            let _ = ORIG_FN.set(std::mem::transmute::<usize, HostFilterTimeFn>(target));
        }

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(true, Ordering::Release);
    log!("[hook] host_filter_time installed at {target:#x}");
}

pub(super) unsafe fn uninstall() {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let (Some(&target), Some(bytes)) = (TARGET.get(), ORIG_BYTES.get()) else {
        return;
    };
    let mut old = 0u32;
    unsafe {
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(false, Ordering::Release);
    log!("[hook] host_filter_time uninstalled");
}
