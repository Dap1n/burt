use std::sync::OnceLock;

use crate::{
    define_get_addr,
    game::{modules, offsets::*},
};

static ACCUMULATOR_ADDR: OnceLock<usize> = OnceLock::new();

define_get_addr!(
    PENTITY_ADDR,
    get_pentity_addr,
    PENTITY_SIG,
    modules::engine(),
);
define_get_addr!(
    KEY_EVENT_ADDR,
    get_key_event_addr,
    KEY_EVENT_SIG,
    modules::engine(),
);
define_get_addr!(
    HOST_FILTER_TIME_ADDR,
    get_host_filter_time_addr,
    HOST_FILTER_TIME_SIG,
    modules::engine(),
);

pub unsafe fn read_abs_addr(addr: usize) -> usize {
    let ptr = addr as *const u32;
    u32::from_le_bytes(unsafe { *(ptr as *const [u8; 4]) }) as usize
}

pub fn get_key_bindings_addr() -> usize {
    let key_event_addr = get_key_event_addr();

    unsafe {
        // Subtract 4 to get the start of the array
        read_abs_addr(key_event_addr + KEY_BINDINGS_OFFSET) - 4
    }
}

pub fn get_filtered_frametime_addr() -> usize {
    let filter_time_addr = get_host_filter_time_addr();

    unsafe {
        // ADDSS instruction: F3 0F 58 05 [ADDR]
        read_abs_addr(filter_time_addr + HOST_FILTERED_FRAMETIME_OFFSET + 4)
    }
}

pub fn get_accumulator_addr() -> usize {
    *ACCUMULATOR_ADDR.get_or_init(|| {
        let instruction_addr = modules::get_addr(modules::engine(), ACCUMULATOR_SIG);
        unsafe { read_abs_addr(instruction_addr + 4) }
    })
}
