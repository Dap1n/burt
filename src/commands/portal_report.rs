use crate::{
    clog,
    game::{entity, math::*, offsets::*, player},
    structs::Color,
};
use burt_command::burt_command;

const PORTAL_SURFACE_DEPTH: f32 = -32.0;

pub unsafe fn resolve_handle(handle: u32) -> *const u8 {
    if handle == 0xFFFF_FFFF {
        return std::ptr::null();
    }
    let index = (handle & 0xFFFF) as i32;
    unsafe {
        let edict = entity::get_edict_of_ent_index(index);
        if edict.is_null() {
            return std::ptr::null();
        }
        *((edict as *const u8).add(EDICT_PORTAL_INTERNAL_OFF) as *const *const u8)
    }
}

pub const unsafe fn read_portal_origin(ent: *const u8) -> Vec3 {
    unsafe {
        (
            *(ent.add(PORTAL_ORIGIN_X) as *const f32),
            *(ent.add(PORTAL_ORIGIN_Y) as *const f32),
            *(ent.add(PORTAL_ORIGIN_Z) as *const f32),
        )
    }
}

pub unsafe fn read_portal_surface(ent: *const u8) -> Vec3 {
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

pub const unsafe fn read_portal_angles(ent: *const u8) -> Vec3 {
    unsafe {
        (
            *(ent.add(PORTAL_PITCH) as *const f32),
            *(ent.add(PORTAL_YAW) as *const f32),
            *(ent.add(PORTAL_ROLL) as *const f32),
        )
    }
}

pub unsafe fn find_linked(current: *const u8) -> *const u8 {
    if current.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let h = *(current.add(PORTAL_H_LINKED) as *const u32);
        resolve_handle(h)
    }
}

/// Reports positions and angles of portals you're currently inside.
///
/// In order for portals to be detected, current player should be in the portal environment.
/// In other words, you should stand near the portal or be save glitched.
#[burt_command]
pub fn burt_portal_report() {
    let current = unsafe { resolve_handle(player::portal_handle()) };
    if current.is_null() {
        clog!(err_prefix;
            "You're not in the portal environment. Stand near the opened portal\n\
            you want the get position of and execute the command"
        );
        return;
    }
    let (x, y, z) = unsafe { read_portal_surface(current) };
    let (pitch, yaw, roll) = unsafe { read_portal_angles(current) };
    clog!(
        [Color::YELLOW => "Current: "],
        [Color::WHITE => "{x} {y} {z} {pitch} {yaw} {roll}"],
    );
    let linked = unsafe { find_linked(current) };
    if linked.is_null() {
        return;
    }
    let (x, y, z) = unsafe { read_portal_surface(linked) };
    let (pitch, yaw, roll) = unsafe { read_portal_angles(linked) };
    clog!(
        [Color::YELLOW => "Linked: "],
        [Color::WHITE => "{x} {y} {z} {pitch} {yaw} {roll}"],
    );
}
