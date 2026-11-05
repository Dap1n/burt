pub mod commands;
pub mod console;
pub mod invoke;
pub mod pe;
pub mod vtable;

use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{bridge::vtable::PluginVTable, clog, create_plugin::create_plugin};

static LOADED: AtomicBool = AtomicBool::new(false);

#[repr(C)]
pub struct MyPlugin {
    vtable: &'static PluginVTable,
}

impl MyPlugin {
    pub const fn new(vtable: &'static PluginVTable) -> Self {
        Self { vtable }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CreateInterface(p_name: *const c_char, p_return_code: *mut c_int) -> *mut c_void {
    let fail = || -> *mut c_void {
        if !p_return_code.is_null() {
            unsafe {
                *p_return_code = 1;
            }
        }
        ptr::null_mut()
    };

    if p_name.is_null() {
        return fail();
    }

    let requested_name = unsafe { CStr::from_ptr(p_name) }
        .to_str()
        .unwrap_or_default();

    if !requested_name.starts_with("ISERVERPLUGINCALLBACKS") {
        return fail();
    }

    if LOADED.swap(true, Ordering::Relaxed) {
        if requested_name.ends_with("003") {
            clog!(err_prefix; "Plugin is already loaded");
        }
        return fail();
    }

    let Some(plugin_ptr) = create_plugin(requested_name) else {
        LOADED.store(false, Ordering::Relaxed);
        return fail();
    };

    if !p_return_code.is_null() {
        unsafe {
            *p_return_code = 0;
        }
    }

    plugin_ptr as *mut c_void
}
