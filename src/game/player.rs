use std::sync::atomic::{AtomicUsize, Ordering};

use crate::game::{entity, math::Vec3, offsets_provider::provider};

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
    let origin_off = provider().player_origin_off();
    unsafe {
        let ox = *(p.add(origin_off) as *const f32);
        let oy = *(p.add(origin_off + 4) as *const f32);
        let oz = *(p.add(origin_off + 8) as *const f32);
        Some((ox, oy, oz))
    }
}

pub fn eye_pos() -> Option<Vec3> {
    let p = ptr();
    if p.is_null() {
        return None;
    }
    let origin_off = provider().player_origin_off();
    unsafe {
        let ox = *(p.add(origin_off) as *const f32);
        let oy = *(p.add(origin_off + 4) as *const f32);
        let oz = *(p.add(origin_off + 8) as *const f32);
        let vz = *(p.add(provider().player_view_offset_off() + 8) as *const f32);
        Some((ox, oy, oz + vz))
    }
}

pub fn velocity() -> Option<Vec3> {
    let p = ptr();
    if p.is_null() {
        return None;
    }
    let velocity_off = provider().player_m_vec_velocity_off();
    unsafe {
        Some((
            *(p.add(velocity_off) as *const f32),
            *(p.add(velocity_off + 4) as *const f32),
            *(p.add(velocity_off + 8) as *const f32),
        ))
    }
}

pub fn surface_friction() -> f32 {
    let p = ptr();
    if p.is_null() {
        return 1.0;
    }

    unsafe { *(p.add(provider().player_surface_friction_off()) as *const f32) }
}

pub fn is_grounded() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }
    unsafe {
        let flags = *(p.add(provider().player_m_flags_off()) as *const u32);
        flags & provider().fl_on_ground() != 0
    }
}

pub fn is_ducked() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }
    unsafe {
        let flags = *(p.add(provider().player_m_flags_off()) as *const u32);
        flags & provider().fl_ducking() != 0
    }
}

pub fn portal_handle() -> u32 {
    let p = ptr();
    if p.is_null() {
        return 0xFFFF_FFFF;
    }
    unsafe { *(p.add(provider().player_h_portal_env_off()) as *const u32) }
}

pub fn is_holding() -> bool {
    let p = ptr();
    if p.is_null() {
        return false;
    }

    let pickup_handle = unsafe { *(p.add(provider().player_m_h_use_entity_off()) as *const u32) };
    pickup_handle != 0xFFFF_FFFF
}

pub unsafe fn get_held_object() -> Option<*const u8> {
    let p = ptr();
    if p.is_null() {
        return None;
    }

    let pickup_handle = unsafe { *(p.add(provider().player_m_h_use_entity_off()) as *const u32) };
    if pickup_handle == 0xFFFF_FFFF {
        return None;
    }

    let (controller_ent, _) = unsafe { entity::resolve_handle(pickup_handle)? };

    let held_obj_handle =
        unsafe { *(controller_ent.add(provider().pickup_m_h_object_off()) as *const u32) };

    if held_obj_handle != 0xFFFF_FFFF
        && held_obj_handle != 0
        && let Some((held_obj_ptr, _)) = unsafe { entity::resolve_handle(held_obj_handle) }
    {
        Some(held_obj_ptr)
    } else {
        None
    }
}
