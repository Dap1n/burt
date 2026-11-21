use std::{
    ffi::{CStr, c_char, c_int},
    mem,
    os::raw::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

use detour::static_detour;

use crate::{
    bridge::vtable::is_simulating,
    engine_build::EngineBuild,
    game::signatures::{self, get_key_bindings_addr},
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

fn dispatch_p2008(p_event: *mut InputEventT) {
    let Some(event) = (unsafe { p_event.as_ref() }) else {
        return;
    };
    let is_pressed = event.m_n_type == 0;

    if should_allow_input(event.m_n_data, is_pressed) {
        unsafe {
            HOOK_P2008.call(p_event);
        }
    }
}

fn dispatch_j2009(a1: u32, keynum: i32, a3: u8, a4: *const c_void) {
    let is_pressed = a3 != 0;

    if should_allow_input(keynum, is_pressed) {
        unsafe {
            HOOK_J2009.call(a1, keynum, a3, a4);
        }
    }
}

pub type KeyEventFnP2008 = unsafe extern "cdecl" fn(*mut InputEventT);
pub type KeyEventFnJ2009 = unsafe extern "cdecl" fn(u32, i32, u8, *const c_void);

static_detour! {
    static HOOK_P2008: unsafe extern "cdecl" fn(*mut InputEventT);
    static HOOK_J2009: unsafe extern "cdecl" fn(u32, i32, u8, *const c_void);
}

pub(super) unsafe fn install(build: EngineBuild) -> Result<(), detour::Error> {
    if build == EngineBuild::Project2008 {
        unsafe {
            let target: KeyEventFnP2008 = mem::transmute(signatures::get_key_event_addr());
            HOOK_P2008.initialize(target, dispatch_p2008)?;
            HOOK_P2008.enable()?;
        }
    } else {
        unsafe {
            let target: KeyEventFnJ2009 = mem::transmute(signatures::get_key_event_addr());
            HOOK_J2009.initialize(target, dispatch_j2009)?;
            HOOK_J2009.enable()?;
        }
    }
    Ok(())
}

pub(super) unsafe fn uninstall(build: EngineBuild) -> Result<(), detour::Error> {
    if build == EngineBuild::Project2008 {
        unsafe {
            HOOK_P2008.disable()?;
        }
    } else {
        unsafe {
            HOOK_J2009.disable()?;
        }
    }
    Ok(())
}

pub fn set_keyinputs_enabled(enabled: bool) {
    IGNORE_INPUTS.store(!enabled, Ordering::Relaxed);
}
