use std::{
    mem,
    os::raw::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering},
    },
};

use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    Memory::{PAGE_EXECUTE_READWRITE, VirtualProtect},
};

use crate::{
    bridge::{commands::CCommand, invoke::get_edict_of_ent_index},
    clog, define_bounds, elog, log,
};
use burt_macros::burt_command;

static RENDER_ACTIVE: AtomicBool = AtomicBool::new(false);
static PHYS_ACTIVE: AtomicBool = AtomicBool::new(false);
static MODE: AtomicI32 = AtomicI32::new(0);

static SERVER_PLAYER: AtomicUsize = AtomicUsize::new(0);
static ORIG_RENDER_VIEW: OnceLock<RenderViewFn> = OnceLock::new();
static PHYS_STATE: OnceLock<PhysState> = OnceLock::new();

const CVIEWRENDER_VTABLE_RVA: usize = 0x3c20c4;
const RENDERVIEW_SLOT: usize = 6;

const CVIEWSETUP_ORIGIN_OFF: usize = 0x2C;
const CVIEWSETUP_ANGLES_OFF: usize = 0x38;

const PHYSICS_SIMULATE_RVA: usize = 0x1c0920;

const SERVER_ORIGIN_OFF: usize = 0x27C;
const SERVER_VIEW_OFFSET_OFF: usize = 0x340;
const PLAYER_H_PORTAL_ENV: usize = 0x1350;
const EDICT_ENTITY_OFF: usize = 0x08;

const PORTAL_ORIGIN_X: usize = 0x1C;
const PORTAL_ORIGIN_Y: usize = 0x20;
const PORTAL_ORIGIN_Z: usize = 0x24;
const PORTAL_PITCH: usize = 0x308;
const PORTAL_YAW: usize = 0x30C;
const PORTAL_ROLL: usize = 0x310;
const PORTAL_H_LINKED: usize = 0x4D0;

const PORTAL_SURFACE_DEPTH: f32 = -32.0;

// Portal aperture + depth of the back-box behind the frame.
// Tuned empirically to match the portal trigger's effective aperture.
const PORTAL_BOX_HALF_WIDTH: f32 = 31.36;
const PORTAL_BOX_HALF_HEIGHT: f32 = 52.92;
const PORTAL_BOX_DEPTH: f32 = 500.0;

const PLAYER_HALF_WIDTH: f32 = 16.0;
const PLAYER_EYE_TO_TOP: f32 = 8.0;

const RENDERVIEW_DRAWVIEWMODEL: i32 = 1;

pub type Vec3 = (f32, f32, f32);
pub type ServerState = (Vec3, f32, Vec3);

type RenderViewFn = unsafe extern "thiscall" fn(
    this: *mut c_void,
    a0: *mut c_void,
    a1: *mut c_void,
    a2: i32,
    a3: i32,
);

type PhysFn = unsafe extern "thiscall" fn(this: *mut c_void);

fn angles_to_forward(pitch_deg: f32, yaw_deg: f32) -> Vec3 {
    let d2r = std::f32::consts::PI / 180.0;
    let (sp, cp) = (pitch_deg * d2r).sin_cos();
    let (sy, cy) = (yaw_deg * d2r).sin_cos();
    (cp * cy, cp * sy, -sp)
}

fn forward_to_angles(fx: f32, fy: f32, fz: f32) -> (f32, f32) {
    let r2d = 180.0 / std::f32::consts::PI;
    let mut yaw = fy.atan2(fx) * r2d;
    if yaw < 0.0 {
        yaw += 360.0;
    }
    let hyp = (fx * fx + fy * fy).sqrt();
    let pitch = -fz.atan2(hyp) * r2d;
    (pitch, yaw)
}

fn angle_vectors_full(pitch: f32, yaw: f32, roll: f32) -> (Vec3, Vec3, Vec3) {
    let d2r = std::f32::consts::PI / 180.0;
    let (sp, cp) = (pitch * d2r).sin_cos();
    let (sy, cy) = (yaw * d2r).sin_cos();
    let (sr, cr) = (roll * d2r).sin_cos();

    let fwd = (cp * cy, cp * sy, -sp);
    let right = (-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp);
    let up = (cr * sp * cy + sr * sy, cr * sp * sy - sr * cy, cr * cp);
    (fwd, right, up)
}

fn angle_matrix(pitch: f32, yaw: f32, roll: f32, pos: Vec3) -> [[f32; 4]; 4] {
    let (fwd, rgt, up) = angle_vectors_full(pitch, yaw, roll);
    [
        [fwd.0, rgt.0, up.0, pos.0],
        [fwd.1, rgt.1, up.1, pos.1],
        [fwd.2, rgt.2, up.2, pos.2],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn matrix_inverse_tr(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let t00 = m[0][0];
    let t01 = m[1][0];
    let t02 = m[2][0];
    let t10 = m[0][1];
    let t11 = m[1][1];
    let t12 = m[2][1];
    let t20 = m[0][2];
    let t21 = m[1][2];
    let t22 = m[2][2];
    let tx = m[0][3];
    let ty = m[1][3];
    let tz = m[2][3];

    let ntx = -(t00 * tx + t01 * ty + t02 * tz);
    let nty = -(t10 * tx + t11 * ty + t12 * tz);
    let ntz = -(t20 * tx + t21 * ty + t22 * tz);

    [
        [t00, t01, t02, ntx],
        [t10, t11, t12, nty],
        [t20, t21, t22, ntz],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let mut s = 0.0f32;
            for k in 0..4 {
                s += a[i][k] * b[k][j];
            }
            out[i][j] = s;
        }
    }
    out
}

fn compute_portal_matrix(
    this_pos: Vec3,
    this_ang: Vec3,
    linked_pos: Vec3,
    linked_ang: Vec3,
) -> [[f32; 4]; 4] {
    let this_to_world = angle_matrix(this_ang.0, this_ang.1, this_ang.2, this_pos);
    let linked_to_world = angle_matrix(linked_ang.0, linked_ang.1, linked_ang.2, linked_pos);
    let this_to_world_inv = matrix_inverse_tr(&this_to_world);

    let rot = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    mat_mul(&mat_mul(&linked_to_world, &rot), &this_to_world_inv)
}

fn apply_portal_matrix(mat: &[[f32; 4]; 4], v: Vec3) -> Vec3 {
    let (x, y, z) = v;
    (
        mat[0][0] * x + mat[0][1] * y + mat[0][2] * z + mat[0][3],
        mat[1][0] * x + mat[1][1] * y + mat[1][2] * z + mat[1][3],
        mat[2][0] * x + mat[2][1] * y + mat[2][2] * z + mat[2][3],
    )
}

fn rotate_vec3(mat: &[[f32; 4]; 4], v: Vec3) -> Vec3 {
    let (x, y, z) = v;
    (
        mat[0][0] * x + mat[0][1] * y + mat[0][2] * z,
        mat[1][0] * x + mat[1][1] * y + mat[1][2] * z,
        mat[2][0] * x + mat[2][1] * y + mat[2][2] * z,
    )
}

fn invert_affine(mat: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut inv = [[0f32; 4]; 4];
    for r in 0..3 {
        for c in 0..3 {
            inv[r][c] = mat[c][r];
        }
    }
    let tx = mat[0][3];
    let ty = mat[1][3];
    let tz = mat[2][3];
    inv[0][3] = -(inv[0][0] * tx + inv[0][1] * ty + inv[0][2] * tz);
    inv[1][3] = -(inv[1][0] * tx + inv[1][1] * ty + inv[1][2] * tz);
    inv[2][3] = -(inv[2][0] * tx + inv[2][1] * ty + inv[2][2] * tz);
    inv[3] = [0.0, 0.0, 0.0, 1.0];
    inv
}

unsafe fn resolve_handle(handle: u32) -> *const u8 {
    if handle == 0xFFFF_FFFF {
        return ptr::null();
    }
    let index = (handle & 0xFFFF) as i32;
    unsafe {
        let edict = get_edict_of_ent_index(index);
        if edict.is_null() {
            return ptr::null();
        }
        *((edict as *const u8).add(EDICT_ENTITY_OFF) as *const *const u8)
    }
}

unsafe fn read_portal_origin(ent: *const u8) -> Vec3 {
    unsafe {
        (
            *(ent.add(PORTAL_ORIGIN_X) as *const f32),
            *(ent.add(PORTAL_ORIGIN_Y) as *const f32),
            *(ent.add(PORTAL_ORIGIN_Z) as *const f32),
        )
    }
}

unsafe fn read_portal_angles(ent: *const u8) -> Vec3 {
    unsafe {
        (
            *(ent.add(PORTAL_PITCH) as *const f32),
            *(ent.add(PORTAL_YAW) as *const f32),
            *(ent.add(PORTAL_ROLL) as *const f32),
        )
    }
}

unsafe fn read_portal_surface(ent: *const u8) -> Vec3 {
    unsafe {
        let (px, py, pz) = read_portal_origin(ent);
        let (pitch, yaw, _) = read_portal_angles(ent);
        let (fx, fy, fz) = angles_to_forward(pitch, yaw);
        (
            px + fx * PORTAL_SURFACE_DEPTH,
            py + fy * PORTAL_SURFACE_DEPTH,
            pz + fz * PORTAL_SURFACE_DEPTH,
        )
    }
}

unsafe fn find_linked(current: *const u8) -> *const u8 {
    if current.is_null() {
        return ptr::null();
    }
    unsafe {
        let h = *(current.add(PORTAL_H_LINKED) as *const u32);
        if h == 0xFFFF_FFFF {
            return ptr::null();
        }
        resolve_handle(h)
    }
}

unsafe fn eye_portal_local(ent: *const u8, eye: Vec3) -> Option<Vec3> {
    if ent.is_null() {
        return None;
    }
    unsafe {
        let surface = read_portal_surface(ent);
        let (pitch, yaw, roll) = read_portal_angles(ent);
        let (fwd, right, up) = angle_vectors_full(pitch, yaw, roll);

        let dx = eye.0 - surface.0;
        let dy = eye.1 - surface.1;
        let dz = eye.2 - surface.2;

        Some((
            fwd.0 * dx + fwd.1 * dy + fwd.2 * dz,
            right.0 * dx + right.1 * dy + right.2 * dz,
            up.0 * dx + up.1 * dy + up.2 * dz,
        ))
    }
}

pub type Range = (f32, f32);
pub type LocalAabb = (Range, Range, Range);

unsafe fn player_hull_local(ent: *const u8, origin: Vec3, view_offset_z: f32) -> Option<LocalAabb> {
    if ent.is_null() {
        return None;
    }
    let surface = unsafe { read_portal_surface(ent) };
    let (pitch, yaw, roll) = unsafe { read_portal_angles(ent) };
    let (fwd, right, up) = angle_vectors_full(pitch, yaw, roll);

    let hx = PLAYER_HALF_WIDTH;
    let hmin = (origin.0 - hx, origin.1 - hx, origin.2);
    let hmax = (
        origin.0 + hx,
        origin.1 + hx,
        origin.2 + view_offset_z + PLAYER_EYE_TO_TOP,
    );

    let mut fmin = f32::MAX;
    let mut fmax = f32::MIN;
    let mut rmin = f32::MAX;
    let mut rmax = f32::MIN;
    let mut umin = f32::MAX;
    let mut umax = f32::MIN;

    for i in 0..8 {
        let x = if i & 1 == 0 { hmin.0 } else { hmax.0 };
        let y = if i & 2 == 0 { hmin.1 } else { hmax.1 };
        let z = if i & 4 == 0 { hmin.2 } else { hmax.2 };
        let dx = x - surface.0;
        let dy = y - surface.1;
        let dz = z - surface.2;
        let lf = fwd.0 * dx + fwd.1 * dy + fwd.2 * dz;
        let lr = right.0 * dx + right.1 * dy + right.2 * dz;
        let lu = up.0 * dx + up.1 * dy + up.2 * dz;
        if lf < fmin {
            fmin = lf;
        }
        if lf > fmax {
            fmax = lf;
        }
        if lr < rmin {
            rmin = lr;
        }
        if lr > rmax {
            rmax = lr;
        }
        if lu < umin {
            umin = lu;
        }
        if lu > umax {
            umax = lu;
        }
    }

    Some(((fmin, fmax), (rmin, rmax), (umin, umax)))
}

unsafe fn should_transform(ent: *const u8, origin: Vec3, view_offset_z: f32, eye: Vec3) -> bool {
    let Some(local_eye) = (unsafe { eye_portal_local(ent, eye) }) else {
        return false;
    };
    let behind = local_eye.0 < 0.0;

    let Some(((fmin, fmax), (rmin, rmax), (umin, umax))) =
        (unsafe { player_hull_local(ent, origin, view_offset_z) })
    else {
        return false;
    };

    let in_shadow = fmax >= -PORTAL_BOX_DEPTH
        && fmin <= 0.0
        && rmax >= -PORTAL_BOX_HALF_WIDTH
        && rmin <= PORTAL_BOX_HALF_WIDTH
        && umax >= -PORTAL_BOX_HALF_HEIGHT
        && umin <= PORTAL_BOX_HALF_HEIGHT;

    behind && !in_shadow
}

unsafe fn read_server_state() -> Option<ServerState> {
    let p = SERVER_PLAYER.load(Ordering::Relaxed) as *const u8;
    if p.is_null() {
        return None;
    }

    let ox = unsafe { *(p.add(SERVER_ORIGIN_OFF) as *const f32) };
    let oy = unsafe { *(p.add(SERVER_ORIGIN_OFF + 4) as *const f32) };
    let oz = unsafe { *(p.add(SERVER_ORIGIN_OFF + 8) as *const f32) };
    let vz = unsafe { *(p.add(SERVER_VIEW_OFFSET_OFF + 8) as *const f32) };

    let eye = (ox, oy, oz + vz);
    Some(((ox, oy, oz), vz, eye))
}

unsafe fn read_portal_handle() -> u32 {
    let p = SERVER_PLAYER.load(Ordering::Relaxed) as *const u8;
    if p.is_null() {
        return 0xFFFF_FFFF;
    }
    unsafe { *(p.add(PLAYER_H_PORTAL_ENV) as *const u32) }
}

fn reconstruct_angles(fwd: Vec3, rgt: Vec3, up: Vec3) -> Vec3 {
    let (pitch, yaw) = forward_to_angles(fwd.0, fwd.1, fwd.2);
    let cp = (pitch.to_radians()).cos();
    if cp.abs() < 0.001 {
        return (pitch, yaw, 0.0);
    }
    let roll = (-rgt.2).atan2(up.2).to_degrees();
    (pitch, yaw, roll)
}

unsafe extern "thiscall" fn hk_render_view(
    this: *mut c_void,
    a0: *mut c_void,
    a1: *mut c_void,
    a2: i32,
    a3: i32,
) {
    let Some(&orig) = ORIG_RENDER_VIEW.get() else {
        return;
    };

    let mode = MODE.load(Ordering::Relaxed);
    if mode == 0 || a0.is_null() {
        unsafe { orig(this, a0, a1, a2, a3) };
        return;
    }

    let Some((origin, vz, eye)) = (unsafe { read_server_state() }) else {
        unsafe { orig(this, a0, a1, a2, a3) };
        return;
    };

    let h = unsafe { read_portal_handle() };
    let current = unsafe { resolve_handle(h) };
    let transform = unsafe { should_transform(current, origin, vz, eye) };

    if h == 0xFFFF_FFFF || !transform || current.is_null() {
        unsafe { orig(this, a0, a1, a2, a3) };
        return;
    }

    let linked = unsafe { find_linked(current) };
    if linked.is_null() {
        unsafe { orig(this, a0, a1, a2, a3) };
        return;
    }

    let mut mat = compute_portal_matrix(
        unsafe { read_portal_surface(current) },
        unsafe { read_portal_angles(current) },
        unsafe { read_portal_surface(linked) },
        unsafe { read_portal_angles(linked) },
    );

    if mode == 2 {
        mat = invert_affine(&mat);
    }

    let f = a0 as *mut f32;
    let (tx, ty, tz) = apply_portal_matrix(&mat, eye);
    unsafe {
        *f.add(CVIEWSETUP_ORIGIN_OFF / 4) = tx;
        *f.add(CVIEWSETUP_ORIGIN_OFF / 4 + 1) = ty;
        *f.add(CVIEWSETUP_ORIGIN_OFF / 4 + 2) = tz;
    }

    let cp = unsafe { *f.add(CVIEWSETUP_ANGLES_OFF / 4) };
    let cy = unsafe { *f.add(CVIEWSETUP_ANGLES_OFF / 4 + 1) };
    let cr = unsafe { *f.add(CVIEWSETUP_ANGLES_OFF / 4 + 2) };

    let (fwd, rgt, up) = angle_vectors_full(cp, cy, cr);
    let fwd_t = rotate_vec3(&mat, fwd);
    let rgt_t = rotate_vec3(&mat, rgt);
    let up_t = rotate_vec3(&mat, up);

    let (np, ny, nr) = reconstruct_angles(fwd_t, rgt_t, up_t);
    unsafe {
        *f.add(CVIEWSETUP_ANGLES_OFF / 4) = np;
        *f.add(CVIEWSETUP_ANGLES_OFF / 4 + 1) = ny;
        *f.add(CVIEWSETUP_ANGLES_OFF / 4 + 2) = nr;
    }

    let out_what_to_draw = a3 & !RENDERVIEW_DRAWVIEWMODEL;

    unsafe {
        orig(this, a0, a1, a2, out_what_to_draw);
    }
}

unsafe fn install_render_hook() {
    if RENDER_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    unsafe {
        let client = GetModuleHandleA(c"client.dll".as_ptr() as *const u8);
        if client.is_null() {
            elog!("[sg_dbg] client.dll not found");
            return;
        }
        let base = client as usize;
        let vtable = (base + CVIEWRENDER_VTABLE_RVA) as *mut *mut c_void;
        let slot = vtable.add(RENDERVIEW_SLOT);

        let _ = ORIG_RENDER_VIEW.set(mem::transmute::<*mut c_void, RenderViewFn>(*slot));

        let mut old = 0u32;
        VirtualProtect(slot as *const c_void, 4, PAGE_EXECUTE_READWRITE, &mut old);
        *slot = hk_render_view as *mut c_void;
        VirtualProtect(slot as *const c_void, 4, old, &mut old);

        RENDER_ACTIVE.store(true, Ordering::SeqCst);
        log!("[sg_dbg] render hook installed");
    }
}

unsafe fn uninstall_render_hook() {
    if !RENDER_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    let Some(&orig) = ORIG_RENDER_VIEW.get() else {
        return;
    };
    unsafe {
        let client = GetModuleHandleA(c"client.dll".as_ptr() as *const u8);
        if client.is_null() {
            return;
        }
        let base = client as usize;
        let vtable = (base + CVIEWRENDER_VTABLE_RVA) as *mut *mut c_void;
        let slot = vtable.add(RENDERVIEW_SLOT);

        let mut old = 0u32;
        VirtualProtect(slot as *const c_void, 4, PAGE_EXECUTE_READWRITE, &mut old);
        *slot = orig as *mut c_void;
        VirtualProtect(slot as *const c_void, 4, old, &mut old);

        RENDER_ACTIVE.store(false, Ordering::SeqCst);
        log!("[sg_dbg] render hook uninstalled");
    }
}

struct PhysState {
    target: usize,
    orig_bytes: [u8; 5],
}

unsafe extern "thiscall" fn hk_physics_simulate(this: *mut c_void) {
    SERVER_PLAYER.store(this as usize, Ordering::Relaxed);
    let Some(state) = PHYS_STATE.get() else {
        return;
    };
    let target = state.target;
    unsafe {
        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        ptr::copy_nonoverlapping(state.orig_bytes.as_ptr(), target as *mut u8, 5);
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        let f: PhysFn = mem::transmute(target);
        f(this);

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (hk_physics_simulate as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);
    }
}

unsafe fn install_physics_hook() {
    if PHYS_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    unsafe {
        let srv = GetModuleHandleA(c"server.dll".as_ptr() as *const u8);
        if srv.is_null() {
            elog!("[sg_dbg] server.dll not found");
            return;
        }
        let target = (srv as usize) + PHYSICS_SIMULATE_RVA;

        if PHYS_STATE.get().is_none() {
            let mut orig_bytes = [0u8; 5];
            ptr::copy_nonoverlapping(target as *const u8, orig_bytes.as_mut_ptr(), 5);
            let _ = PHYS_STATE.set(PhysState { target, orig_bytes });
        }

        let mut old = 0u32;
        VirtualProtect(target as *const c_void, 5, PAGE_EXECUTE_READWRITE, &mut old);
        *(target as *mut u8) = 0xE9;
        *((target + 1) as *mut i32) =
            (hk_physics_simulate as *const () as usize).wrapping_sub(target + 5) as i32;
        VirtualProtect(target as *const c_void, 5, old, &mut old);

        PHYS_ACTIVE.store(true, Ordering::SeqCst);
        log!("[sg_dbg] physics hook installed at {target:#x}");
    }
}

unsafe fn uninstall_physics_hook() {
    if !PHYS_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    let Some(state) = PHYS_STATE.get() else {
        return;
    };
    unsafe {
        let mut old = 0u32;
        VirtualProtect(
            state.target as *const c_void,
            5,
            PAGE_EXECUTE_READWRITE,
            &mut old,
        );
        ptr::copy_nonoverlapping(state.orig_bytes.as_ptr(), state.target as *mut u8, 5);
        VirtualProtect(state.target as *const c_void, 5, old, &mut old);

        PHYS_ACTIVE.store(false, Ordering::SeqCst);
        log!("[sg_dbg] physics hook uninstalled");
    }
}

/// Save Glitch debug — override camera to see how portals will land
///
///   -1 = unload (restore original vtable + function bytes)
///    0 = normal camera (hooks stay installed, transform disabled)
///    1 = ghost — transform through the current portal to the linked portal
///    2 = ghost — inverted matrix (from linked to current)
#[burt_command]
pub fn burt_sg_dbg(mode: i32) {
    define_bounds!(-1 <= mode <= 2);

    if mode == -1 {
        unsafe {
            uninstall_render_hook();
            uninstall_physics_hook();
        }
        MODE.store(0, Ordering::Relaxed);
        log!("[sg_dbg] unloaded");
        return;
    }

    unsafe {
        install_render_hook();
        install_physics_hook();
    }
    MODE.store(mode, Ordering::Relaxed);
}
