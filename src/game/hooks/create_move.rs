use std::{
    mem,
    os::raw::c_void,
    sync::atomic::{AtomicU64, Ordering},
};

use detour::static_detour;

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
        provider::provider,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMoveArgs {
    pub this: *mut c_void,
    pub sequence_number: i32,
    pub active: u8,
}

define_subscriptions!(CreateMoveArgs);

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

    if engine_build::current_build() == EngineBuild::Project2008 {
        unsafe {
            HOOK_P2008.call(this, frametime, active);
        }
    } else {
        unsafe {
            HOOK_J2009.call(this, sequence_number, frametime, active);
        }
    }
}

fn dispatch_p2008(this: *mut c_void, frametime: f32, active: u8) {
    dispatch_inner(this, 0, frametime, active);
}

fn dispatch_j2009(this: *mut c_void, sequence_number: i32, frametime: f32, active: u8) {
    dispatch_inner(this, sequence_number, frametime, active);
}

pub type CreateMoveFnP2008 = unsafe extern "thiscall" fn(*mut c_void, f32, u8);
pub type CreateMoveFnJ2009 = unsafe extern "thiscall" fn(*mut c_void, i32, f32, u8);

static_detour! {
    static HOOK_P2008: unsafe extern "thiscall" fn(*mut c_void, f32, u8);
    static HOOK_J2009: unsafe extern "thiscall" fn(*mut c_void, i32, f32, u8);
}

pub(super) unsafe fn install(build: EngineBuild) -> Result<(), detour::Error> {
    let addr = modules::get_addr(modules::client(), provider().cinput_create_move_sig());
    if build == EngineBuild::Project2008 {
        unsafe {
            let target: CreateMoveFnP2008 = mem::transmute(addr);
            HOOK_P2008.initialize(target, dispatch_p2008)?;
            HOOK_P2008.enable()?;
        }
    } else {
        unsafe {
            let target: CreateMoveFnJ2009 = mem::transmute(addr);
            HOOK_J2009.initialize(target, dispatch_j2009)?;
            HOOK_J2009.enable()?;
        }
    }
    Ok(())
}

pub(super) unsafe fn uninstall(build: EngineBuild) -> Result<(), detour::Error> {
    if let Ok(mut subs) = SUBS.try_lock() {
        subs.clear();
    }
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
