use std::{mem, os::raw::c_void, sync::atomic::Ordering};

use detour::static_detour;

use crate::{
    commands::{bvars::BVAR_TAS_SKIP_TO, tas},
    define_subscriptions,
    game::{math::Vec3, modules, provider::provider},
};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderViewArgs {
    pub this: *mut c_void,
    pub setup: *mut c_void,
    pub a1: *mut c_void,
    pub a2: i32,
    pub a3: i32,
}

define_subscriptions!(RenderViewArgs);

fn dispatch(this: *mut c_void, setup: *mut c_void, a1: *mut c_void, a2: i32, a3: i32) {
    if tas::SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        let skip_to = BVAR_TAS_SKIP_TO.load(Ordering::Relaxed);
        if skip_to != 0 && tas::TICK.load(Ordering::Relaxed) <= skip_to {
            return;
        }
    }

    let mut args = RenderViewArgs {
        this,
        setup,
        a1,
        a2,
        a3,
    };

    if !this.is_null() {
        let subs = SUBS.lock().unwrap().clone();
        subs.into_iter().for_each(|cmd| cmd.get_fn()(&mut args));
    }

    unsafe {
        HOOK.call(this, setup, a1, a2, a3);
    }
}

pub type RenderViewFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut c_void, *mut c_void, i32, i32);

static_detour! {
    static HOOK: unsafe extern "thiscall" fn(*mut c_void, *mut c_void, *mut c_void, i32, i32);
}

pub(super) unsafe fn install() -> Result<(), detour::Error> {
    let addr = modules::get_addr(modules::client(), provider().cviewrender_render_view_sig());
    unsafe {
        let target: RenderViewFn = mem::transmute(addr);
        HOOK.initialize(target, dispatch)?;
        HOOK.enable()?;
    }
    Ok(())
}

pub(super) unsafe fn uninstall() -> Result<(), detour::Error> {
    if let Ok(mut subs) = SUBS.try_lock() {
        subs.clear();
    }
    unsafe {
        HOOK.disable()?;
    }
    Ok(())
}

pub unsafe fn read_setup_origin(setup: *const f32) -> Vec3 {
    let origin_off = provider().cviewsetup_origin_off();
    unsafe {
        (
            *setup.add(origin_off / 4),
            *setup.add(origin_off / 4 + 1),
            *setup.add(origin_off / 4 + 2),
        )
    }
}
