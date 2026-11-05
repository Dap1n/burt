use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{bridge::MyPlugin, consts::DESC};

static SIMULATING: AtomicBool = AtomicBool::new(false);

pub fn is_simulating() -> bool {
    SIMULATING.load(Ordering::Relaxed)
}

#[repr(C)]
pub struct PluginVTable {
    pub load: extern "thiscall" fn(*mut MyPlugin, *const c_void, *const c_void) -> bool,
    pub unload: extern "thiscall" fn(*mut MyPlugin),
    pub pause: extern "thiscall" fn(*mut MyPlugin),
    pub un_pause: extern "thiscall" fn(*mut MyPlugin),
    pub get_plugin_description: extern "thiscall" fn(*mut MyPlugin) -> *const c_char,
    pub level_init: extern "thiscall" fn(*mut MyPlugin, *const c_char),
    pub server_activate: extern "thiscall" fn(*mut MyPlugin, *mut c_void, c_int, c_int),
    pub game_frame: extern "thiscall" fn(*mut MyPlugin, bool),
    pub level_shutdown: extern "thiscall" fn(*mut MyPlugin),
    pub client_active: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    pub client_disconnect: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    pub client_put_in_server: extern "thiscall" fn(*mut MyPlugin, *mut c_void, *const c_char),
    pub set_command_client: extern "thiscall" fn(*mut MyPlugin, c_int),
    pub client_settings_changed: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    pub client_connect: extern "thiscall" fn(
        *mut MyPlugin,
        *mut bool,
        *mut c_void,
        *const c_char,
        *const c_char,
        *mut c_char,
        c_int,
    ) -> c_int,
    pub client_command: extern "thiscall" fn(*mut MyPlugin, *mut c_void, *const c_void) -> c_int,
    pub network_id_validated:
        extern "thiscall" fn(*mut MyPlugin, *const c_char, *const c_char) -> c_int,
    pub on_query_cvar_value_finished: extern "thiscall" fn(
        *mut MyPlugin,
        c_int,
        *mut c_void,
        c_int,
        *const c_char,
        *const c_char,
    ),
    pub on_edict_allocated: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    pub on_edict_freed: extern "thiscall" fn(*mut MyPlugin, *const c_void),
}

pub extern "thiscall" fn load(
    _: *mut MyPlugin,
    interface_factory: *const c_void,
    _: *const c_void,
) -> bool {
    crate::vtable::load(interface_factory)
}

pub extern "thiscall" fn unload(_: *mut MyPlugin) {
    crate::vtable::unload();
}

pub extern "thiscall" fn level_init(_: *mut MyPlugin, p_map_name: *const c_char) {
    let Ok(map_name) = unsafe { CStr::from_ptr(p_map_name) }.to_str() else {
        return;
    };
    crate::vtable::level_init(map_name);
}

pub extern "thiscall" fn get_plugin_description(_: *mut MyPlugin) -> *const c_char {
    DESC.as_ptr()
}

pub extern "thiscall" fn game_frame(_: *mut MyPlugin, simulating: bool) {
    SIMULATING.store(simulating, Ordering::Relaxed);
    crate::vtable::game_frame();
}

pub extern "thiscall" fn dummy_void(_: *mut MyPlugin) {}
pub extern "thiscall" fn dummy_void_ptr(_: *mut MyPlugin, _: *mut c_void) {}
pub extern "thiscall" fn dummy_void_3(_: *mut MyPlugin, _: *mut c_void, _: c_int, _: c_int) {}

pub extern "thiscall" fn dummy_void_2(_: *mut MyPlugin, _: *mut c_void, _: *const c_char) {}
pub extern "thiscall" fn dummy_void_int(_: *mut MyPlugin, _: c_int) {}

pub extern "thiscall" fn dummy_int_6(
    _: *mut MyPlugin,
    _: *mut bool,
    _: *mut c_void,
    _: *const c_char,
    _: *const c_char,
    _: *mut c_char,
    _: c_int,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_3_ptr(
    _: *mut MyPlugin,
    _: *mut c_void,
    _: *const c_void,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_2c(
    _: *mut MyPlugin,
    _: *const c_char,
    _: *const c_char,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_int_3(
    _: *mut MyPlugin,
    _: c_int,
    _: *mut c_void,
    _: c_int,
    _: *const c_char,
    _: *const c_char,
) {
}

pub extern "thiscall" fn dummy_void_ptr_const(_: *mut MyPlugin, _: *const c_void) {}
