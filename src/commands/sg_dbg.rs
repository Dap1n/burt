use std::sync::atomic::{AtomicU8, Ordering};

use crate::{
    commands::portal_report::{
        find_linked, read_portal_angles, read_portal_surface, resolve_handle,
    },
    define_bounds,
    game::{
        hooks::{
            Subscriber,
            render_view::{self, RenderViewArgs},
        },
        math::*,
        offsets::*,
        player,
    },
};
use burt_command::burt_command;

static MODE: AtomicU8 = AtomicU8::new(0);

unsafe fn portal_local(ent: *const u8, pos: Vec3) -> Option<Vec3> {
    let surface = unsafe { read_portal_surface(ent) };
    let (pitch, yaw, roll) = unsafe { read_portal_angles(ent) };
    let (fwd, right, up) = angle_vectors_full(pitch, yaw, roll);

    let dx = pos.0 - surface.0;
    let dy = pos.1 - surface.1;
    let dz = pos.2 - surface.2;

    Some((
        fwd.0 * dx + fwd.1 * dy + fwd.2 * dz,
        right.0 * dx + right.1 * dy + right.2 * dz,
        up.0 * dx + up.1 * dy + up.2 * dz,
    ))
}

unsafe fn should_transform(setup: *const f32, eye: Vec3) -> bool {
    let Some((vel_x, vel_y, vel_z)) = player::velocity() else {
        return false;
    };
    let (sx, sy, sz) = unsafe { render_view::read_setup_origin(setup) };
    let dx = sx - eye.0;
    let dy = sy - eye.1;
    let dz = sz - eye.2;
    let speed_sq = vel_x.powi(2) + vel_y.powi(2) + vel_z.powi(2);
    let threshold = (speed_sq * 0.02 * 0.02 * 4.0).max(20.0);
    dx.powi(2) + dy.powi(2) + dz.powi(2) < threshold
}

fn on_render(args: &mut RenderViewArgs) {
    let mode = MODE.load(Ordering::Relaxed);
    if mode == 0 || args.setup.is_null() {
        return;
    }

    let Some(eye) = player::eye_pos() else {
        return;
    };

    let current = unsafe { resolve_handle(player::portal_handle()) };
    if current.is_null() {
        return;
    }

    let Some((local_eye_depth, _, _)) = (unsafe { portal_local(current, eye) }) else {
        return;
    };
    if local_eye_depth > 0.0 {
        return;
    }

    if unsafe { !should_transform(args.setup as *const f32, eye) } != (mode == 2) {
        return;
    }

    let linked = unsafe { find_linked(current) };
    if linked.is_null() {
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

    let input_pos = if mode == 1 {
        eye
    } else {
        unsafe { render_view::read_setup_origin(args.setup as *const f32) }
    };

    let f = args.setup as *mut f32;

    let (tx, ty, tz) = apply_portal_matrix(&mat, input_pos);
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

    args.a3 &= !RENDERVIEW_DRAWVIEWMODEL;
}

/// Overrides camera to visualize ghost/body during Save Glitch.
///
/// `mode:` debugging mode:
///   `0` — normal: disables override, restores default engine camera
///   `1` — ghost: forces the camera to show the "ghost" view.
///     If the engine is already rendering this view (when behind a portal, for instance), it does nothing.
///     If the engine is showing the body view, it applies the forward portal matrix to simulate the ghost view
///   `2` — body: forces the camera to show the actual player body position.
///     If the engine is rendering the native ghost view, it applies the inverse portal matrix to undo the warp and snap back to the body.
///     If the engine is already showing the body view, it does nothing
#[burt_command(cheat)]
pub fn burt_sg_dbg(mode: u32) {
    define_bounds!(0 <= mode <= 2);

    if mode == 0 {
        render_view::unsubscribe(Subscriber::SgDbg(on_render));
    } else {
        render_view::subscribe(Subscriber::SgDbg(on_render));
    }

    MODE.store(mode as u8, Ordering::Relaxed);
}
