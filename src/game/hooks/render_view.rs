use std::{
    mem,
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    commands::{bvars::BVAR_TAS_SKIP_TO, tas},
    define_subscriptions,
    game::{math::Vec3, modules, offsets_provider::provider},
    log,
    os::*,
};

pub type RenderViewFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut c_void, *mut c_void, i32, i32);

#[derive(Debug, Clone, PartialEq)]
pub struct RenderViewArgs {
    pub this: *mut c_void,
    pub setup: *mut c_void,
    pub a1: *mut c_void,
    pub a2: i32,
    pub a3: i32,
}

define_subscriptions!(RenderViewArgs);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGET: OnceLock<usize> = OnceLock::new();
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static ORIG_FN: OnceLock<RenderViewFn> = OnceLock::new();

unsafe extern "thiscall" fn dispatch(
    this: *mut c_void,
    setup: *mut c_void,
    a1: *mut c_void,
    a2: i32,
    a3: i32,
) {
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

    let (Some(&target), Some(bytes), Some(&orig)) = (TARGET.get(), ORIG_BYTES.get(), ORIG_FN.get())
    else {
        return;
    };

    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        orig(args.this, args.setup, args.a1, args.a2, args.a3);

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

    let target = modules::get_addr(modules::client(), provider().cviewrender_render_view_sig());

    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);
            let _ = ORIG_FN.set(mem::transmute::<usize, RenderViewFn>(target));
        }

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }

    INSTALLED.store(true, Ordering::Release);
    log!("[hook] render_view installed at {target:#x}");
}

pub(super) unsafe fn uninstall() {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let (Some(&target), Some(bytes)) = (TARGET.get(), ORIG_BYTES.get()) else {
        return;
    };

    if let Ok(mut subs) = SUBS.try_lock() {
        subs.clear();
    }

    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }

    INSTALLED.store(false, Ordering::Release);
    log!("[hook] render_view uninstalled");
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
