use std::{mem, os::raw::c_void};

use detour::static_detour;

use crate::game::{modules, player, provider::provider};

fn dispatch(this: *mut c_void) {
    player::set(this as usize);
    unsafe {
        HOOK.call(this);
    }
}

type PhysicsFn = unsafe extern "thiscall" fn(*mut c_void);

static_detour! {
    static HOOK: unsafe extern "thiscall" fn(*mut c_void);
}

pub(super) unsafe fn install() -> Result<(), detour::Error> {
    let addr = modules::get_addr(modules::server(), provider().physics_simulate_sig());
    unsafe {
        let target: PhysicsFn = mem::transmute(addr);
        HOOK.initialize(target, dispatch)?;
        HOOK.enable()?;
    }
    Ok(())
}

#[inline]
pub(super) unsafe fn uninstall() -> Result<(), detour::Error> {
    unsafe {
        HOOK.disable()?;
    }
    Ok(())
}
