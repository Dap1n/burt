use std::{
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::game::{entity, math::Vec3, provider::provider, signatures};

pub static PLAYER: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub fn set(ptr: usize) {
    PLAYER.store(ptr, Ordering::Relaxed);
}

#[inline]
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

type FnScrollScale = unsafe extern "thiscall" fn(this: *const u8, direction: i32);

pub fn execute_scaling(mut scale: i32) -> bool {
    if scale == 0 {
        return false;
    }
    let p = ptr();
    if p.is_null() {
        return false;
    }

    let weapon_handle = unsafe { *(p.add(provider().player_m_h_active_weapon()) as *const u32) };
    if weapon_handle == 0xFFFF_FFFF || weapon_handle == 0 {
        return false;
    }

    let Some((weapon_ent, _)) = (unsafe { entity::resolve_handle(weapon_handle) }) else {
        return false;
    };

    let scale_down = scale < 0;
    if scale_down {
        scale = -scale;
    }
    scale = scale.min(32);
    let direction = if scale_down { 1 } else { -1 };
    unsafe {
        let scroll_scale: FnScrollScale = mem::transmute(signatures::get_scroll_scale_addr());
        (0..scale).for_each(|_| scroll_scale(weapon_ent, direction))
    }
    true
}

type FnNextPhoto = unsafe extern "cdecl" fn();

#[inline]
pub fn next_photo() {
    unsafe {
        let next_photo_fn: FnNextPhoto = mem::transmute(signatures::get_next_photo_addr());
        next_photo_fn();
    }
}

type FnSelectPhoto = unsafe extern "cdecl" fn(n: i32);

#[inline]
pub fn select_photo(n: i32) {
    unsafe {
        let select_photo_fn: FnSelectPhoto = mem::transmute(signatures::get_select_photo_addr());
        select_photo_fn(n);
    }
}

#[derive(Debug, Clone)]
pub struct CaptureReport {
    pub entity: *const u8,
    pub in_placement: bool,
    pub current_step: i32,
    pub multiplier: f32,
    pub min_bound: i32,
    pub max_bound: i32,
}

type FnGetBound = unsafe extern "thiscall" fn(this: *const u8, tracker: *const u8) -> i32;
type FnGetScaleFloat =
    unsafe extern "thiscall" fn(this: *const u8, step: i32, tracker: *const u8) -> f32;

pub fn get_capture_report() -> Option<CaptureReport> {
    let weapon_ent = {
        let p = ptr();
        if p.is_null() {
            return None;
        }

        let weapon_handle =
            unsafe { *(p.add(provider().player_m_h_active_weapon()) as *const u32) };
        if weapon_handle == 0xFFFF_FFFF || weapon_handle == 0 {
            return None;
        }
        unsafe { entity::resolve_handle(weapon_handle)?.0 }
    };

    let current_step = unsafe { *(weapon_ent.add(provider().entity_step_off()) as *const i32) };

    let (multiplier, min_bound, max_bound) = {
        let controller_ptr =
            unsafe { *(weapon_ent.add(provider().entity_controller_off()) as *const *const u8) };
        if controller_ptr.is_null() {
            return None;
        }

        let vtable = unsafe { *(controller_ptr as *const *const usize) };
        if vtable.is_null() {
            return None;
        }

        let scale_func_ptr = unsafe { *vtable.add(provider().controller_scale_func_slot()) };
        let min_func_ptr = unsafe { *vtable.add(provider().controller_min_func_slot()) };
        let max_func_ptr = unsafe { *vtable.add(provider().controller_max_func_slot()) };

        if min_func_ptr == 0 || max_func_ptr == 0 || scale_func_ptr == 0 {
            return None;
        }

        let get_scale: FnGetScaleFloat = unsafe { std::mem::transmute(scale_func_ptr) };
        let get_min: FnGetBound = unsafe { std::mem::transmute(min_func_ptr) };
        let get_max: FnGetBound = unsafe { std::mem::transmute(max_func_ptr) };

        let tracker_arg = unsafe { weapon_ent.add(provider().entity_tracker_off()) };

        let scale = unsafe { get_scale(controller_ptr, current_step, tracker_arg) };
        let min = unsafe { -get_min(controller_ptr, tracker_arg) };
        let max = unsafe { get_max(controller_ptr, tracker_arg) };
        (scale, min, max)
    };

    let entity = {
        let captured_handle =
            unsafe { *(weapon_ent.add(provider().entity_captured_handle_off()) as *const u32) };
        if captured_handle == 0xFFFF_FFFF || captured_handle == 0 {
            return None;
        }
        unsafe { entity::resolve_handle(captured_handle)?.0 }
    };

    let in_placement =
        unsafe { *(weapon_ent.add(provider().entity_in_placement_off()) as *const i32) != 0 };

    Some(CaptureReport {
        entity,
        in_placement,
        current_step,
        multiplier,
        min_bound,
        max_bound,
    })
}
