#![allow(clippy::too_many_arguments)]

use std::{
    mem,
    os::raw::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

use detour::static_detour;

use crate::{
    commands::run::sr_stop,
    engine_build::EngineBuild,
    game::{entity, modules, provider::provider},
};

pub static OBSERVING: AtomicBool = AtomicBool::new(false);

fn dispatch(
    this: *mut c_void,
    input_name: *const u8,
    activator: *mut c_void,
    caller: *mut c_void,
    value: *const u8,
    param_6: u32,
    param_7: u32,
    param_8: u32,
    param_9: i32,
    param_10: u32,
) {
    if OBSERVING.load(Ordering::Relaxed) {
        let name = unsafe { entity::entity_name(this as *const u8) };
        if name.is_some_and(|s| s.starts_with("p2008_end")) {
            OBSERVING.store(false, Ordering::Relaxed);
            sr_stop::stop_internal(true);
        }
    }

    unsafe {
        HOOK.call(
            this, input_name, activator, caller, value, param_6, param_7, param_8, param_9,
            param_10,
        );
    }
}

pub type AcceptInputFn = unsafe extern "thiscall" fn(
    this: *mut c_void,
    input_name: *const u8,
    activator: *mut c_void,
    caller: *mut c_void,
    value: *const u8,
    param_6: u32,
    param_7: u32,
    param_8: u32,
    param_9: i32,
    param_10: u32,
);

static_detour! {
    static HOOK: unsafe extern "thiscall" fn(
        *mut c_void,
        *const u8,
        *mut c_void,
        *mut c_void,
        *const u8,
        u32,
        u32,
        u32,
        i32,
        u32,
    );
}

pub(super) unsafe fn install(build: EngineBuild) -> Result<(), detour::Error> {
    if build != EngineBuild::Project2008 {
        return Ok(());
    }
    let addr = modules::get_addr(modules::server(), provider().accept_input_sig());
    unsafe {
        let target: AcceptInputFn = mem::transmute(addr);
        HOOK.initialize(target, dispatch)?;
        HOOK.enable()?;
    }
    Ok(())
}

pub(super) unsafe fn uninstall(build: EngineBuild) -> Result<(), detour::Error> {
    if build == EngineBuild::Project2008 {
        unsafe {
            HOOK.disable()?;
        }
    }
    Ok(())
}
