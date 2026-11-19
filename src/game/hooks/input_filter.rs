use std::{
    ffi::{CStr, c_char, c_int},
    mem,
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    bridge::vtable::is_simulating,
    engine_build::{self, EngineBuild},
    game::signatures::{get_key_bindings_addr, get_key_event_addr},
    log,
    os::*,
};

// Used by Project 2008 (Alien Swarm code)
#[derive(Debug)]
#[repr(C)]
pub struct InputEventT {
    pub m_n_type: c_int, // 0 = Pressed, 1 = Released, etc.
    pub m_n_tick: c_int,
    pub m_n_data: c_int, // The key code
    pub m_n_data_2: c_int,
    pub m_n_data_3: c_int,
}

pub type KeyEventFnP2008 = unsafe extern "cdecl" fn(*mut InputEventT);
pub type KeyEventFnJ2009 = unsafe extern "cdecl" fn(u32, i32, u8, *const c_void);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static TARGET: OnceLock<usize> = OnceLock::new();

static ORIG_FN_P2008: OnceLock<KeyEventFnP2008> = OnceLock::new();
static ORIG_FN_J2009: OnceLock<KeyEventFnJ2009> = OnceLock::new();

static IGNORE_INPUTS: AtomicBool = AtomicBool::new(false);

#[inline]
fn should_allow_input(keynum: i32, is_pressed: bool) -> bool {
    if !IGNORE_INPUTS.load(Ordering::Relaxed)
        || !is_pressed
        || !is_simulating()
        || [57, 70].contains(&keynum)
    {
        return true;
    }

    let addr = get_key_bindings_addr() + (keynum as usize * 8);
    let binding_ptr = unsafe { *(addr as *const *const c_char) };

    let Some(binding) = (!binding_ptr.is_null())
        .then(|| unsafe { CStr::from_ptr(binding_ptr) }.to_str().ok())
        .flatten()
        .map(|s| s.trim().to_lowercase())
    else {
        return false;
    };

    ["toggleconsole", "cancelselect", "pause", "unpause"].contains(&binding.as_str())
        || binding.starts_with("burt_tas_")
}

unsafe extern "cdecl" fn hooked_key_event_p2008(p_event: *mut InputEventT) {
    let Some(event) = (unsafe { p_event.as_ref() }) else {
        return;
    };

    let is_pressed = event.m_n_type == 0;

    if should_allow_input(event.m_n_data, is_pressed)
        && let Some(&target) = TARGET.get()
        && let Some(bytes) = ORIG_BYTES.get()
    {
        unsafe {
            let mut old = 0u32;
            VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
            ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
            VirtualProtect(target as *const c_void, 5, old, &mut old);

            if let Some(orig) = ORIG_FN_P2008.get() {
                orig(p_event);
            }

            VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
            *(target as *mut u8) = 0xE9;
            *((target + 1) as *mut i32) =
                (hooked_key_event_p2008 as *const () as usize).wrapping_sub(target + 5) as i32;
            VirtualProtect(target as *const c_void, 5, old, &mut old);
        }
    }
}

unsafe extern "cdecl" fn hooked_key_event_j2009(a1: u32, keynum: i32, a3: u8, a4: *const c_void) {
    let is_pressed = a3 != 0;

    if should_allow_input(keynum, is_pressed)
        && let Some(&target) = TARGET.get()
        && let Some(bytes) = ORIG_BYTES.get()
    {
        unsafe {
            let mut old = 0u32;
            VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
            ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
            VirtualProtect(target as *const c_void, 5, old, &mut old);

            if let Some(orig) = ORIG_FN_J2009.get() {
                orig(a1, keynum, a3, a4);
            }

            VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
            *(target as *mut u8) = 0xE9;
            *((target + 1) as *mut i32) =
                (hooked_key_event_j2009 as *const () as usize).wrapping_sub(target + 5) as i32;
            VirtualProtect(target as *const c_void, 5, old, &mut old);
        }
    }
}

pub(super) unsafe fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let target = get_key_event_addr();
    let build = engine_build::current_build();
    let mut old = 0u32;

    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);

            if build == EngineBuild::Project2008 {
                let _ = ORIG_FN_P2008.set(mem::transmute::<usize, KeyEventFnP2008>(target));
            } else {
                let _ = ORIG_FN_J2009.set(mem::transmute::<usize, KeyEventFnJ2009>(target));
            }
        }

        let hook_addr = if build == EngineBuild::Project2008 {
            hooked_key_event_p2008 as *const () as usize
        } else {
            hooked_key_event_j2009 as *const () as usize
        };

        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) = (hook_addr).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(true, Ordering::Release);
    log!("[hook] input filter installed at {target:#x}");
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
    log!("[hook] input filter uninstalled");
}

pub fn set_keyinputs_enabled(enabled: bool) {
    IGNORE_INPUTS.store(!enabled, Ordering::Relaxed);
}
