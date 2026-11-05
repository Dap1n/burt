use std::{
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    game::{modules, offsets::*, player},
    log,
    os::*,
};

type PhysicsFn = unsafe extern "thiscall" fn(*mut c_void);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGET: OnceLock<usize> = OnceLock::new();
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static ORIG_FN: OnceLock<PhysicsFn> = OnceLock::new();

unsafe extern "thiscall" fn dispatch(this: *mut c_void) {
    player::set(this as usize);

    let (Some(&target), Some(bytes), Some(&orig)) = (TARGET.get(), ORIG_BYTES.get(), ORIG_FN.get())
    else {
        return;
    };

    let mut old = 0u32;
    unsafe {
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        orig(this);

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
}

pub(super) unsafe fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let target = modules::get_addr(modules::server(), PHYSICS_SIMULATE_SIG);
    let mut old = 0u32;
    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);
            let _ = ORIG_FN.set(std::mem::transmute::<usize, PhysicsFn>(target));
        }

        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(true, Ordering::Release);
    log!("[hook] physics installed at {target:#x}");
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
    log!("[hook] physics uninstalled");
}
