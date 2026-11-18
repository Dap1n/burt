use std::{
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use portable_atomic::AtomicF32;

use crate::{
    bridge::vtable::is_simulating,
    commands::{
        j2009::bvars::BVAR_TAS_QUANTIZE_MOVEMENT,
        tas::{self, Point2D},
    },
    define_subscriptions,
    engine_build::{self, EngineBuild},
    game::{
        cvars::get_move_speed, hooks::create_move::SCHEDULED_ANGLES, math, modules,
        provider::provider,
    },
    log,
    os::*,
};

pub type ProcessCmdFn = unsafe extern "thiscall" fn(*mut c_void, *mut u8);

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessCmdArgs {
    pub this: *mut c_void,
    pub cmd: *mut u8,
}

define_subscriptions!(ProcessCmdArgs);

static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGET: OnceLock<usize> = OnceLock::new();
static ORIG_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static ORIG_FN: OnceLock<ProcessCmdFn> = OnceLock::new();

pub static SCHEDULED_MOVEMENT: AtomicU64 = AtomicU64::new(0);

unsafe extern "thiscall" fn dispatch(this: *mut c_void, cmd: *mut u8) {
    cache_body_angles(cmd as *const u8);

    let mut args = ProcessCmdArgs { this, cmd };
    let seq = unsafe { *(args.cmd.add(provider().cusercmd_sequence_number_off()) as *const i32) };

    if seq != 0 && is_simulating() && !this.is_null() {
        SCHEDULED_MOVEMENT.store(0, Ordering::Relaxed);
        let tas_running = tas::SESSION_COUNTER.load(Ordering::Relaxed) == 1;
        if tas_running {
            tas::on_process_cmd();
        }
        let subs = SUBS.lock().unwrap().clone();
        subs.into_iter().for_each(|cmd| cmd.get_fn()(&mut args));
        let mut movement = Point2D::from_bits(SCHEDULED_MOVEMENT.load(Ordering::Relaxed));
        movement.x = math::snap(movement.x, 1e-6);
        movement.y = math::snap(movement.y, 1e-6);
        if tas_running {
            tas::on_process_cmd_end(movement);
        }
        process_movement(&mut movement);
        if !movement.is_zero() {
            unsafe {
                *(args.cmd.add(provider().cusercmd_side_off()) as *mut f32) = movement.x;
                *(args.cmd.add(provider().cusercmd_forward_off()) as *mut f32) = movement.y;
            }
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

        orig(args.this, args.cmd);

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

    let target = modules::get_addr(modules::client(), provider().chl_input_process_cmd_sig());

    unsafe {
        if let Ok(mut subs) = SUBS.try_lock() {
            subs.clear();
        }

        if ORIG_BYTES.get().is_none() {
            let mut bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), 5);
            let _ = ORIG_BYTES.set(bytes);
            let _ = TARGET.set(target);
            let _ = ORIG_FN.set(std::mem::transmute::<usize, ProcessCmdFn>(target));
        }

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (dispatch as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
    INSTALLED.store(true, Ordering::Release);
    log!("[hook] process_cmd installed at {target:#x}");
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
    log!("[hook] process_cmd uninstalled");
}

static CACHED_BODY_PITCH: AtomicF32 = AtomicF32::new(0.0);
static CACHED_BODY_YAW: AtomicF32 = AtomicF32::new(0.0);

fn cache_body_angles(cmd: *const u8) {
    unsafe {
        let pitch = *(cmd.add(provider().cusercmd_pitch_off()) as *const f32);
        let yaw = *(cmd.add(provider().cusercmd_yaw_off()) as *const f32);
        CACHED_BODY_PITCH.store(pitch, Ordering::Relaxed);
        CACHED_BODY_YAW.store(yaw, Ordering::Relaxed);
    }
}

pub fn write_body_angles(pitch: f32, yaw: f32) {
    CACHED_BODY_PITCH.store(pitch, Ordering::Relaxed);
    CACHED_BODY_YAW.store(yaw, Ordering::Relaxed);
}

pub fn cached_body_angles() -> (f32, f32) {
    (
        CACHED_BODY_PITCH.load(Ordering::Relaxed),
        CACHED_BODY_YAW.load(Ordering::Relaxed),
    )
}

pub fn schedule_movement(target_x: f32, target_y: f32) {
    let point = Point2D {
        x: target_x,
        y: target_y,
    };
    SCHEDULED_MOVEMENT.store(point.to_bits(), Ordering::Relaxed);
}

fn process_movement(movement: &mut Point2D) {
    let move_speed = get_move_speed();
    if movement.y > 0.0 {
        movement.y *= move_speed.forward;
    } else {
        movement.y *= move_speed.back;
    }
    movement.x *= move_speed.side;
    if engine_build::current_build() != EngineBuild::Project2008
        && BVAR_TAS_QUANTIZE_MOVEMENT.load(Ordering::Relaxed)
        && SCHEDULED_ANGLES.load(Ordering::Relaxed) != 0
    {
        (movement.x, movement.y) = quantize_movement(movement.x, movement.y);
    }
}

const CL_SPEED: f32 = 450.0;
const JOYSTICK_MAX: f32 = 32768.0;
const ANALOG_STEP: f32 = CL_SPEED / JOYSTICK_MAX;
const CLAMP_THRESHOLD: f32 = 175.0;

/// Quantizes an analog movement vector to simulate 16-bit XInput hardware.
///
/// If the vector's length is >= 175.0, it is first scaled to exactly 450.0
/// (simulating a full stick deflection) and then quantized. This ensures we
/// get maximum angular precision and perfectly trigger the engine's f32 clamp.
///
/// If the vector's length is < 175.0, it is quantized directly to the nearest
/// multiple of (450.0 / 32768.0), simulating partial stick tilt.
fn quantize_movement(mut x: f32, mut y: f32) -> (f32, f32) {
    let length_sq = x * x + y * y;

    if length_sq >= CLAMP_THRESHOLD * CLAMP_THRESHOLD {
        let scale = CL_SPEED / length_sq.sqrt();
        x *= scale;
        y *= scale;
    }

    let raw_x = (x / ANALOG_STEP).round();
    let raw_y = (y / ANALOG_STEP).round();

    let clamped_x = raw_x.clamp(-32768.0, 32767.0);
    let clamped_y = raw_y.clamp(-32768.0, 32767.0);

    let qx = clamped_x * ANALOG_STEP;
    let qy = clamped_y * ANALOG_STEP;

    (qx, qy)
}
