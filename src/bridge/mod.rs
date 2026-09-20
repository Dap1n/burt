pub mod commands;
pub mod console;
pub mod invoke;
pub mod vtable;

use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
    ptr,
};

use crate::{bridge::vtable::PluginVTable, create_plugin::create_plugin};

#[repr(C)]
pub struct MyPlugin {
    vtable: &'static PluginVTable,
}

impl MyPlugin {
    pub fn new(vtable: &'static PluginVTable) -> Self {
        Self { vtable }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CreateInterface(p_name: *const c_char, p_return_code: *mut c_int) -> *mut c_void {
    if p_name.is_null() {
        if !p_return_code.is_null() {
            unsafe {
                *p_return_code = 1; // IFACE_FAILED
            }
        }
        return ptr::null_mut();
    }

    let requested_name = unsafe { CStr::from_ptr(p_name) }
        .to_str()
        .unwrap_or_default();

    match create_plugin(requested_name) {
        Some(plugin_ptr) => {
            if !p_return_code.is_null() {
                unsafe {
                    *p_return_code = 0; // IFACE_OK
                }
            }
            plugin_ptr as *mut c_void
        }
        None => {
            if !p_return_code.is_null() {
                unsafe {
                    *p_return_code = 1; // IFACE_FAILED
                }
            }
            ptr::null_mut()
        }
    }
}
