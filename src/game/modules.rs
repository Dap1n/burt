use std::{
    ffi::{CStr, c_char},
    sync::atomic::{AtomicUsize, Ordering},
};

use pe_sigscan::{count_in_text, find_in_text};

use crate::os::GetModuleHandleA;

pub static ENGINE_BASE: AtomicUsize = AtomicUsize::new(0);
pub static CLIENT_BASE: AtomicUsize = AtomicUsize::new(0);
pub static SERVER_BASE: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn fetch(name: &CStr) -> usize {
    unsafe { GetModuleHandleA(name.as_ptr() as *const c_char) as usize }
}

pub fn init() {
    ENGINE_BASE.store(fetch(c"engine.dll"), Ordering::Relaxed);
    CLIENT_BASE.store(fetch(c"client.dll"), Ordering::Relaxed);
    SERVER_BASE.store(fetch(c"server.dll"), Ordering::Relaxed);
}

#[inline(always)]
pub fn engine() -> usize {
    ENGINE_BASE.load(Ordering::Relaxed)
}

#[inline(always)]
pub fn client() -> usize {
    CLIENT_BASE.load(Ordering::Relaxed)
}

#[inline(always)]
pub fn server() -> usize {
    SERVER_BASE.load(Ordering::Relaxed)
}

pub fn get_addr(module_base: usize, pattern: &[Option<u8>]) -> usize {
    match count_in_text(module_base, pattern) {
        1 => find_in_text(module_base, pattern).unwrap(),
        n => {
            panic!("pattern matched {n} sites; module_base=0x{module_base:X}; pattern={pattern:?}");
        }
    }
}
