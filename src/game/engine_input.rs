use std::{mem, os::raw::c_void};

use crate::game::signatures::get_key_event_addr;

type KeyEventFn = unsafe extern "cdecl" fn(u32, i32, u8, *const c_void);

#[inline]
fn get_fn() -> KeyEventFn {
    unsafe { mem::transmute::<usize, KeyEventFn>(get_key_event_addr()) }
}

pub unsafe fn press_button(keynum: i32) {
    let mut buf = [0, 0, 0, 0, 61, 28, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0];
    buf[8..12].copy_from_slice(&(keynum as u32).to_le_bytes());
    unsafe {
        get_fn()(0, keynum, 1, buf.as_ptr() as *const c_void);
    }
}

pub unsafe fn release_button(keynum: i32) {
    let mut buf = [1, 0, 0, 0, 61, 28, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0];
    buf[8..12].copy_from_slice(&(keynum as u32).to_le_bytes());
    unsafe {
        get_fn()(0, keynum, 0, buf.as_ptr() as *const c_void);
    }
}
