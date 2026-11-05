use std::sync::atomic::{AtomicUsize, Ordering};

use crate::game::entity;

use super::{math::Vec3, offsets::*};

pub const TICKTIME: f32 = 1.0 / 60.0;

pub static PLAYER: AtomicUsize = AtomicUsize::new(0);

pub fn set(ptr: usize) {
    PLAYER.store(ptr, Ordering::Relaxed);
}

pub fn ptr() -> *const u8 {
    PLAYER.load(Ordering::Relaxed) as *const u8
}

pub fn origin_pos() -> Option<Vec3> {
    let p = ptr();
    if p.is_null() {
        return None;
    }
    unsafe {
        let ox = *(p.add(PLAYER_ORIGIN_OFF) as *const f32);
        let oy = *(p.add(PLAYER_ORIGIN_OFF + 4) as *const f32);
        let oz = *(p.add(PLAYER_ORIGIN_OFF + 8) as *const f32);
        Some((ox, oy, oz))
    }
}

pub fn eye_pos() -> Option<Vec3> {
    let p = ptr();
    if p.is_null() {
        return None;
    }
    unsafe {
        let ox = *(p.add(PLAYER_ORIGIN_OFF) as *const f32);
        let oy = *(p.add(PLAYER_ORIGIN_OFF + 4) as *const f32);
        let oz = *(p.add(PLAYER_ORIGIN_OFF + 8) as *const f32);
        let vz = *(p.add(PLAYER_VIEW_OFFSET_OFF + 8) as *const f32);
        Some((ox, oy, oz + vz))
    }
}

pub fn velocity() -> Option<Vec3> {
    let p = ptr();
    if p.is_null() {
        return None;
    }
    unsafe {
        Some((
            *(p.add(PLAYER_M_VEC_VELOCITY_OFF) as *const f32),
            *(p.add(PLAYER_M_VEC_VELOCITY_OFF + 4) as *const f32),
            *(p.add(PLAYER_M_VEC_VELOCITY_OFF + 8) as *const f32),
        ))
    }
}

pub fn surface_friction() -> f32 {
    let p = ptr();
    if p.is_null() {
        return 1.0;
    }
    unsafe { *(p.add(PLAYER_SURFACE_FRICTION_OFF) as *const f32) }
}

pub fn is_grounded() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }
    unsafe {
        let flags = *(p.add(PLAYER_M_FLAGS_OFF) as *const u32);
        flags & FL_ONGROUND != 0
    }
}

pub fn is_ducked() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }
    unsafe {
        let flags = *(p.add(PLAYER_M_FLAGS_OFF) as *const u32);
        flags & FL_DUCKING != 0
    }
}

pub fn portal_handle() -> u32 {
    let p = ptr();
    if p.is_null() {
        return 0xFFFF_FFFF;
    }
    unsafe { *(p.add(PLAYER_H_PORTAL_ENV_OFF) as *const u32) }
}

pub fn is_holding() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }

    let pickup_handle = unsafe { *(p.add(PLAYER_M_H_USE_ENTITY_OFF) as *const u32) };
    pickup_handle != 0xFFFF_FFFF
}

pub unsafe fn get_held_object() -> Option<*const u8> {
    let p = ptr();
    if p.is_null() {
        return None;
    }

    let pickup_handle = unsafe { *(p.add(PLAYER_M_H_USE_ENTITY_OFF) as *const u32) };
    if pickup_handle == 0xFFFF_FFFF {
        return None;
    }

    let (controller_ent, _) = unsafe { entity::resolve_handle(pickup_handle)? };

    let held_obj_handle = unsafe { *(controller_ent.add(PICKUP_M_H_OBJECT_OFF) as *const u32) };

    if held_obj_handle != 0xFFFF_FFFF
        && held_obj_handle != 0
        && let Some((held_obj_ptr, _)) = unsafe { entity::resolve_handle(held_obj_handle) }
    {
        Some(held_obj_ptr)
    } else {
        None
    }
}
