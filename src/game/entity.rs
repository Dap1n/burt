use std::{
    ffi::CStr,
    mem,
    os::raw::{c_char, c_void},
};

use crate::game::{math::Vec3, provider::provider, signatures::get_pentity_addr};

#[repr(C)]
pub struct edict_t {
    flags: i32,
    area: i16,
    headnode: i16,
    free_time: f32,
    networkable: *const c_void,
    unknown: *const c_void,
    pv_private_data: *mut c_void,
}

const NUM_ENT_ENTRY_BITS: u32 = 12;
const INVALID_EHANDLE_INDEX: u32 = 0xFFFF_FFFF;
const INVALID_ENTRY_INDEX: u32 = 0xFFF;
const MAX_EDICTS: u32 = 2048;

pub unsafe fn resolve_handle(handle: u32) -> Option<(*const u8, u32)> {
    if handle == INVALID_EHANDLE_INDEX {
        return None;
    }

    let entry = handle & ((1 << NUM_ENT_ENTRY_BITS) - 1);
    if entry == INVALID_ENTRY_INDEX || entry >= MAX_EDICTS {
        return None;
    }

    let serial = handle >> NUM_ENT_ENTRY_BITS;
    let (_, ent) = unsafe { edict_entity_by_index(entry as i32)? };
    let ent_ref_handle = unsafe { *(ent.add(provider().entity_m_ref_ehandle_off()) as *const u32) };
    let ent_serial = ent_ref_handle >> NUM_ENT_ENTRY_BITS;
    (ent_serial == serial).then_some((ent, serial))
}

pub unsafe fn get_edict_of_ent_index(ent_index: i32) -> *mut edict_t {
    unsafe {
        type FnPEntityOfEntIndex = unsafe extern "cdecl" fn(i32) -> *mut edict_t;
        let p_entity_fn: FnPEntityOfEntIndex = mem::transmute(get_pentity_addr());
        p_entity_fn(ent_index)
    }
}

pub unsafe fn edict_entity_by_index(index: i32) -> Option<(*mut edict_t, *const u8)> {
    let edict = unsafe { get_edict_of_ent_index(index) };
    if edict.is_null() {
        return None;
    }
    let ent = unsafe { entity_from_edict(edict) };
    (!ent.is_null()).then_some((edict, ent))
}

pub unsafe fn edict_entity_by_name(name: &str, index: i32) -> Option<(*mut edict_t, *const u8)> {
    let mut counter = 0;
    for i in 1..2048 {
        let Some((edict, ent)) = (unsafe { edict_entity_by_index(i) }) else {
            continue;
        };

        if unsafe { entity_name(ent) }.as_deref() == Some(name)
            || unsafe { entity_classname(ent) }.as_deref() == Some(name)
        {
            if counter == index {
                return Some((edict, ent));
            }
            counter += 1;
        }
    }
    None
}

#[inline]
pub unsafe fn entity_from_edict(edict: *const edict_t) -> *const u8 {
    unsafe {
        *((edict as *const c_void as *const u8).add(provider().edict_base_entity_off())
            as *const *const u8)
    }
}

pub unsafe fn entity_origin(ent: *const u8) -> Option<Vec3> {
    let origin_off = provider().entity_origin_off();
    unsafe {
        Some((
            *(ent.add(origin_off) as *const f32),
            *(ent.add(origin_off + 4) as *const f32),
            *(ent.add(origin_off + 8) as *const f32),
        ))
    }
}

unsafe fn read_cstr_field(ent: *const u8, offset: usize) -> Option<String> {
    let ptr = unsafe { *(ent.add(offset) as *const *const c_char) };

    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

#[inline]
pub unsafe fn entity_name(ent: *const u8) -> Option<String> {
    unsafe { read_cstr_field(ent, provider().entity_name_off()) }
}

#[inline]
pub unsafe fn entity_classname(ent: *const u8) -> Option<String> {
    unsafe { read_cstr_field(ent, provider().entity_classname_off()) }
}
