use std::sync::OnceLock;

use crate::{
    define_get_addr,
    game::{modules, offsets_provider::provider},
};

static ACCUMULATOR_ADDR: OnceLock<usize> = OnceLock::new();

define_get_addr!(
    PENTITY_ADDR,
    get_pentity_addr,
    p_entity_sig,
    modules::engine(),
);
define_get_addr!(
    KEY_EVENT_ADDR,
    get_key_event_addr,
    key_event_sig,
    modules::engine(),
);
define_get_addr!(
    HOST_FILTER_TIME_ADDR,
    get_host_filter_time_addr,
    host_filter_time_sig,
    modules::engine(),
);

pub unsafe fn read_abs_addr(addr: usize) -> usize {
    let ptr = addr as *const u32;
    u32::from_le_bytes(unsafe { *(ptr as *const [u8; 4]) }) as usize
}

pub fn get_key_bindings_addr() -> usize {
    let key_event_addr = get_key_event_addr();
    unsafe {
        read_abs_addr(key_event_addr + provider().key_bindings_offset())
            - provider().key_bindings_base_offset()
    }
}

pub fn get_filtered_frametime_addr() -> usize {
    let filter_time_addr = get_host_filter_time_addr();

    unsafe {
        // ADDSS instruction: F3 0F 58 05 [ADDR]
        read_abs_addr(filter_time_addr + provider().host_filtered_frametime_offset() + 4)
    }
}

pub fn get_accumulator_addr() -> usize {
    *ACCUMULATOR_ADDR.get_or_init(|| {
        let instruction_addr = modules::get_addr(modules::engine(), provider().accumulator_sig());
        unsafe { read_abs_addr(instruction_addr + 4) }
    })
}
