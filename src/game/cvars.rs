use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    bridge::commands::ICVAR,
    engine_build::{self, EngineBuild},
    game::offsets_provider::provider,
};

const NOT_FOUND: usize = usize::MAX;

type SetStrFn = unsafe extern "thiscall" fn(*mut c_void, *const c_char);
type SetF32Fn = unsafe extern "thiscall" fn(*mut c_void, f32);
type SetI32Fn = unsafe extern "thiscall" fn(*mut c_void, i32);

#[inline]
unsafe fn vtable_slot(obj: usize, row: usize) -> *const c_void {
    unsafe {
        let vt = *(obj as *const *const *const c_void);
        *vt.add(row)
    }
}

pub struct CachedConVar {
    name: &'static CStr,
    ptr: AtomicUsize,
}

impl CachedConVar {
    pub const fn new(name: &'static CStr) -> Self {
        Self {
            name,
            ptr: AtomicUsize::new(0),
        }
    }

    fn resolve(&self) -> usize {
        let cached = self.ptr.load(Ordering::Relaxed);
        if cached == NOT_FOUND {
            return 0;
        }
        if cached != 0 {
            return cached;
        }

        let cvar = ICVAR.load(Ordering::Relaxed);
        if cvar == 0 {
            return 0;
        }

        let vtable = unsafe { *(cvar as *const *const *const c_void) };
        let raw = unsafe { *vtable.add(provider().icvar_find_command_slot()) };
        if raw.is_null() {
            return 0;
        }

        let find_fn: extern "thiscall" fn(*mut c_void, *const c_char) -> *mut c_void =
            unsafe { std::mem::transmute(raw) };

        let obj = find_fn(cvar as *mut c_void, self.name.as_ptr() as *const c_char) as usize;

        if obj == 0 {
            self.ptr.store(NOT_FOUND, Ordering::Relaxed);
        } else {
            self.ptr.store(obj, Ordering::Relaxed);
        }
        obj
    }

    #[inline]
    pub fn float_or(&self, default: f32) -> f32 {
        let ptr = self.resolve();
        if ptr == 0 {
            return default;
        }

        unsafe { *((ptr + provider().convar_float_value_off()) as *const f32) }
    }

    #[inline]
    pub fn int_or(&self, default: i32) -> i32 {
        let ptr = self.resolve();
        if ptr == 0 {
            return default;
        }

        unsafe { *((ptr + provider().convar_int_value_off()) as *const i32) }
    }

    pub fn set_string(&self, val: &str) {
        let obj = self.resolve();
        if obj == 0 {
            return;
        }
        if let Ok(cs) = CString::new(val) {
            unsafe {
                let f: SetStrFn =
                    std::mem::transmute(vtable_slot(obj, provider().convar_set_string_row()));
                f(obj as *mut c_void, cs.as_ptr());
            }
        }
    }

    pub fn set_float(&self, val: f32) {
        if val.round() == val {
            self.set_int(val as i32);
            return;
        }
        let obj = self.resolve();
        if obj != 0 {
            unsafe {
                let f: SetF32Fn =
                    std::mem::transmute(vtable_slot(obj, provider().convar_set_float_row()));
                f(obj as *mut c_void, val);
            }
        }
    }

    pub fn set_int(&self, val: i32) {
        let obj = self.resolve();
        if obj != 0 {
            unsafe {
                let f: SetI32Fn =
                    std::mem::transmute(vtable_slot(obj, provider().convar_set_int_row()));
                f(obj as *mut c_void, val);
            }
        }
    }
}

pub static SV_CHEATS: CachedConVar = CachedConVar::new(c"sv_cheats");
pub static SV_FRICTION: CachedConVar = CachedConVar::new(c"sv_friction");
pub static SV_STOPSPEED: CachedConVar = CachedConVar::new(c"sv_stopspeed");
pub static SV_ACCELERATE: CachedConVar = CachedConVar::new(c"sv_accelerate");
pub static SV_MAXSPEED: CachedConVar = CachedConVar::new(c"sv_maxspeed");
pub static SV_MAXVELOCITY: CachedConVar = CachedConVar::new(c"sv_maxvelocity");

pub static CL_MOUSEENABLE: CachedConVar = CachedConVar::new(c"cl_mouseenable");
pub static CL_INTERPOLATE: CachedConVar = CachedConVar::new(c"cl_interpolate");
pub static CL_PITCHUP: CachedConVar = CachedConVar::new(c"cl_pitchup");
pub static CL_PITCHDOWN: CachedConVar = CachedConVar::new(c"cl_pitchdown");
pub static CL_FORWARDSPEED: CachedConVar = CachedConVar::new(c"cl_forwardspeed");
pub static CL_SIDESPEED: CachedConVar = CachedConVar::new(c"cl_sidespeed");
pub static CL_BACKSPEED: CachedConVar = CachedConVar::new(c"cl_backspeed");
pub static FPS_MAX: CachedConVar = CachedConVar::new(c"fps_max");
pub static HOST_FRAMERATE: CachedConVar = CachedConVar::new(c"host_framerate");
pub static SENSITIVITY: CachedConVar = CachedConVar::new(c"sensitivity");
pub static JOYSTICK: CachedConVar = CachedConVar::new(c"joystick");
pub static M_YAW: CachedConVar = CachedConVar::new(c"m_yaw");
pub static M_PITCH: CachedConVar = CachedConVar::new(c"m_pitch");
pub static MAT_MOTION_BLUR_ENABLED: CachedConVar = CachedConVar::new(c"mat_motion_blur_enabled");

#[inline]
pub fn cheats_enabled() -> bool {
    SV_CHEATS.int_or(0) != 0
}

#[inline]
pub fn set_mouseenable(enable: bool) {
    CL_MOUSEENABLE.set_int(enable as i32);
}

#[derive(Debug, Clone, Copy)]
pub struct MoveSpeed {
    pub forward: f32,
    pub side: f32,
    pub back: f32,
}

impl MoveSpeed {
    pub fn with_ones() -> Self {
        Self {
            forward: 1.0,
            side: 1.0,
            back: 1.0,
        }
    }
}

pub fn get_move_speed() -> MoveSpeed {
    let speed = if engine_build::current_build() == EngineBuild::Project2008 {
        275.0
    } else {
        175.0
    };
    MoveSpeed {
        forward: CL_FORWARDSPEED.float_or(speed).clamp(0.0, speed),
        side: CL_SIDESPEED.float_or(speed).clamp(0.0, speed),
        back: CL_BACKSPEED.float_or(speed).clamp(0.0, speed),
    }
}

#[inline]
pub fn set_mat_motion_blur(enable: bool) {
    MAT_MOTION_BLUR_ENABLED.set_int(enable as i32);
}

#[inline]
pub fn set_cl_interpolate(enable: bool) {
    CL_INTERPOLATE.set_int(enable as i32);
}

#[inline]
pub fn set_joystick(enable: bool) {
    JOYSTICK.set_int(enable as i32);
}

#[inline]
pub fn deg_per_count_yaw() -> f32 {
    let res = SENSITIVITY.float_or(3.0) * M_YAW.float_or(0.022);
    if res != 0.0 { res } else { 0.001 }
}

#[inline]
pub fn deg_per_count_pitch() -> f32 {
    let res = SENSITIVITY.float_or(3.0) * M_PITCH.float_or(0.022);
    if res != 0.0 { res } else { 0.001 }
}
