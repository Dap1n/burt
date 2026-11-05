use std::{
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
    game::{
        cinput,
        cvars::{CL_PITCHDOWN, CL_PITCHUP, deg_per_count_pitch, deg_per_count_yaw},
        hooks::process_cmd,
        math, modules,
        offsets::*,
    },
    log,
    os::*,
};

pub type CreateMoveFn = unsafe extern "thiscall" fn(*mut c_void, i32, f32, u8);

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMoveArgs {
    pub this: *mut c_void,
    pub sequence_number: i32,
    pub frametime: f32,
    pub active: u8,
}

define_subscriptions!(CreateMoveArgs);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static TARGET: OnceLock<usize> = OnceLock::new();
static ORIG_FN: OnceLock<CreateMoveFn> = OnceLock::new();

pub static SCHEDULED_ANGLES: AtomicU64 = AtomicU64::new(u64::MAX);

unsafe extern "thiscall" fn dispatch(
    this: *mut c_void,
    sequence_number: i32,
    frametime: f32,
    active: u8,
) {
    let mut args = CreateMoveArgs {
        this,
        sequence_number,
        frametime,
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

    let (Some(&target), Some(bytes), Some(&orig)) = (TARGET.get(), ORIG_BYTES.get(), ORIG_FN.get())
    else {
        return;
    };

    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        orig(args.this, args.sequence_number, args.frametime, args.active);

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
    let target = modules::get_addr(modules::client(), CINPUT_CREATE_MOVE_SIG);
    unsafe {
        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);
            let _ = ORIG_FN.set(std::mem::transmute::<usize, CreateMoveFn>(target));
        }

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
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
