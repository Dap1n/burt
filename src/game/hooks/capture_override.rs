use std::{mem, os::raw::c_void, sync::atomic::Ordering};

use detour::static_detour;

use crate::{
    commands::p2008::bvars::BVAR_CAPTURE_OVERRIDE,
    engine_build::EngineBuild,
    game::{modules, provider::provider},
};

#[derive(Debug, Clone, Copy)]
enum CaptureOverride {
    Preserved,
    Forced(bool),
    Inverted,
}

impl CaptureOverride {
    fn from_index(index: i8) -> Self {
        match index {
            -1 => Self::Preserved,
            0 => Self::Forced(false),
            1 => Self::Forced(true),
            2 => Self::Inverted,
            _ => unreachable!(),
        }
    }
}

fn dispatch(this: *mut c_void) -> u8 {
    match CaptureOverride::from_index(BVAR_CAPTURE_OVERRIDE.load(Ordering::Relaxed)) {
        CaptureOverride::Preserved => unsafe { HOOK.call(this) },
        CaptureOverride::Forced(false) => 0,
        CaptureOverride::Forced(true) => 1,
        CaptureOverride::Inverted => unsafe { (HOOK.call(this) == 0) as u8 },
    }
}

pub type CanBeCapturedFn = unsafe extern "thiscall" fn(this: *mut c_void) -> u8;

static_detour! {
    static HOOK: unsafe extern "thiscall" fn(*mut c_void) -> u8;
}

pub(super) unsafe fn install(build: EngineBuild) -> Result<(), detour::Error> {
    if build != EngineBuild::Project2008 {
        return Ok(());
    }
    let addr = modules::get_addr(modules::server(), provider().can_be_captured_sig());
    unsafe {
        let target: CanBeCapturedFn = mem::transmute(addr);
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
