use std::{
    ffi::CStr,
    mem,
    os::raw::{c_char, c_void},
};

use crate::{
    game::{math::Vec3, offsets::*, signatures::get_pentity_addr},
    os::*,
};

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

    let ent_ref_handle = unsafe { *(ent.add(ENTITY_M_REF_EHANDLE_OFF) as *const u32) };
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

fn mem_ok(addr: usize, len: usize) -> bool {
    if !(0x10000..0x8000_0000).contains(&addr) {
        return false;
    }
    unsafe {
        let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();
        let ret = VirtualQuery(
            addr as *const c_void,
            &mut mbi,
            mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        if ret == 0
            || mbi.state != MEM_COMMIT
            || mbi.protect & PAGE_GUARD != 0
            || mbi.protect & PAGE_NOACCESS != 0
        {
            return false;
        }
        addr + len <= (mbi.base_address as usize) + mbi.region_size
    }
}

pub unsafe fn edict_entity_by_index(index: i32) -> Option<(*mut edict_t, *const u8)> {
    let edict = unsafe { get_edict_of_ent_index(index) };
    if edict.is_null() {
        return None;
    }
    let ent = unsafe { entity_from_edict(edict) };
    if ent.is_null() {
        None
    } else {
        Some((edict, ent))
    }
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
pub const unsafe fn entity_from_edict(edict: *const edict_t) -> *const u8 {
    unsafe {
        *((edict as *const c_void as *const u8).add(EDICT_BASE_ENTITY_OFF) as *const *const u8)
    }
}

pub unsafe fn entity_origin(ent: *const u8) -> Option<Vec3> {
    if !mem_ok(ent as usize + ENTITY_ORIGIN_OFF, 12) {
        return None;
    }
    unsafe {
        Some((
            *(ent.add(ENTITY_ORIGIN_OFF) as *const f32),
            *(ent.add(ENTITY_ORIGIN_OFF + 4) as *const f32),
            *(ent.add(ENTITY_ORIGIN_OFF + 8) as *const f32),
        ))
    }
}

unsafe fn read_cstr_field(ent: *const u8, offset: usize) -> Option<String> {
    if !mem_ok(ent as usize + offset, 4) {
        return None;
    }

    let ptr = unsafe { *(ent.add(offset) as *const *const c_char) };

    if ptr.is_null() {
        return None;
    }

    if !mem_ok(ptr as usize, 2) {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

#[inline]
pub unsafe fn entity_name(ent: *const u8) -> Option<String> {
    unsafe { read_cstr_field(ent, ENTITY_NAME_OFF) }
}

#[inline]
pub unsafe fn entity_classname(ent: *const u8) -> Option<String> {
    unsafe { read_cstr_field(ent, ENTITY_CLASSNAME_OFF) }
}
