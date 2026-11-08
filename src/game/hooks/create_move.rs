use std::{
    mem,
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use crate::{
    bridge::vtable::is_simulating,
    commands::tas::{self, CURRENT_DX, CURRENT_DY, Point2D},
    define_subscriptions,
    engine_build::{self, EngineBuild},
    game::{
        cinput,
        cvars::{CL_PITCHDOWN, CL_PITCHUP, deg_per_count_pitch, deg_per_count_yaw},
        hooks::process_cmd,
        math, modules,
        offsets_provider::provider,
    },
    log,
    os::*,
};

pub type CreateMoveFnP2008 = unsafe extern "thiscall" fn(*mut c_void, i32, u8);
pub type CreateMoveFnJ2009 = unsafe extern "thiscall" fn(*mut c_void, i32, f32, u8);

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMoveArgs {
    pub this: *mut c_void,
    pub sequence_number: i32,
    pub active: u8,
}

define_subscriptions!(CreateMoveArgs);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static TARGET: OnceLock<usize> = OnceLock::new();

static ORIG_FN_P2008: OnceLock<CreateMoveFnP2008> = OnceLock::new();
static ORIG_FN_J2009: OnceLock<CreateMoveFnJ2009> = OnceLock::new();

pub static SCHEDULED_ANGLES: AtomicU64 = AtomicU64::new(u64::MAX);

#[inline]
fn dispatch_inner(this: *mut c_void, sequence_number: i32, frametime: f32, active: u8) {
    let mut args = CreateMoveArgs {
        this,
        sequence_number,
        active,
    };

    if is_simulating() && !this.is_null() {
        let (current_pitch, current_yaw) = process_cmd::cached_body_angles();
        SCHEDULED_ANGLES.store(u64::MAX, Ordering::Relaxed);
        if tas::SESSION_COUNTER.load(Ordering::Relaxed) != 0 {
            tas::on_create_move(&mut args);
        }
        let subs = SUBS.lock().unwrap().clone();
        subs.into_iter().for_each(|cmd| cmd.get_fn()(&mut args));
        let angles = SCHEDULED_ANGLES.load(Ordering::Relaxed);
        if angles != u64::MAX {
            let point = Point2D::from_bits(angles);
            aim_mouse(&args, point, current_pitch, current_yaw);
        }
    }

    let (Some(&target), Some(bytes)) = (TARGET.get(), ORIG_BYTES.get()) else {
        return;
    };

    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        let build = engine_build::current_build();
        if build == EngineBuild::Project2008 {
            if let Some(orig) = ORIG_FN_P2008.get() {
                orig(args.this, args.sequence_number, args.active);
            }
        } else if let Some(orig) = ORIG_FN_J2009.get() {
            orig(args.this, args.sequence_number, frametime, args.active);
        }

        let dispatch_addr = if build == EngineBuild::Project2008 {
            dispatch_p2008 as *const () as usize
        } else {
            dispatch_j2009 as *const () as usize
        };

        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) = (dispatch_addr).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
}

unsafe extern "thiscall" fn dispatch_p2008(this: *mut c_void, sequence_number: i32, active: u8) {
    dispatch_inner(this, sequence_number, 0.0, active);
}

unsafe extern "thiscall" fn dispatch_j2009(
    this: *mut c_void,
    sequence_number: i32,
    frametime: f32,
    active: u8,
) {
    dispatch_inner(this, sequence_number, frametime, active);
}

pub(super) unsafe fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }

    let target = modules::get_addr(modules::client(), provider().cinput_create_move_sig());
    let build = engine_build::current_build();

    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);

            if build == EngineBuild::Project2008 {
                let _ = ORIG_FN_P2008.set(mem::transmute(target));
            } else {
                let _ = ORIG_FN_J2009.set(mem::transmute(target));
            }
        }

        let dispatch_addr = if build == EngineBuild::Project2008 {
            dispatch_p2008 as *const () as usize
        } else {
            dispatch_j2009 as *const () as usize
        };

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) = (dispatch_addr).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(true, Ordering::Release);
    log!("[hook] create_move installed at {target:#x}");
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
    let mut old = 0u32;
    unsafe {
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(false, Ordering::Release);
    log!("[hook] create_move uninstalled");
}

pub fn schedule_angles(target_pitch: f32, target_yaw: f32) {
    let point = Point2D {
        x: target_yaw,
        y: target_pitch,
    };
    SCHEDULED_ANGLES.store(point.to_bits(), Ordering::Relaxed);
    process_cmd::write_body_angles(target_pitch, target_yaw);
}

fn aim_mouse(args: &CreateMoveArgs, mut point: Point2D, current_pitch: f32, current_yaw: f32) {
    point.y = point
        .y
        .clamp(-CL_PITCHDOWN.float_or(89.0), CL_PITCHUP.float_or(89.0));
    let d_yaw = math::shortest_dyaw(point.x - current_yaw);
    let d_pitch = point.y - current_pitch;

    CURRENT_DX.store(math::snap(-d_yaw, 1e-6), Ordering::Relaxed);
    CURRENT_DY.store(math::snap(-d_pitch, 1e-6), Ordering::Relaxed);

    let dx_counts = -d_yaw / deg_per_count_yaw();
    let dy_counts = d_pitch / deg_per_count_pitch();

    unsafe {
        cinput::inject_mouse_delta(args.this as *mut u8, dx_counts, dy_counts);
    }
}
