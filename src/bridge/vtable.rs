use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
};

use crate::{bridge::MyPlugin, statics::DESC};

#[repr(C)]
pub struct PluginVTable {
    // 0x00 (Slot 0)
    pub load: extern "thiscall" fn(*mut MyPlugin, *const c_void, *const c_void) -> bool,
    // 0x04 (Slot 1)
    pub unload: extern "thiscall" fn(*mut MyPlugin),
    // 0x08 (Slot 2)
    pub pause: extern "thiscall" fn(*mut MyPlugin),
    // 0x0C (Slot 3)
    pub un_pause: extern "thiscall" fn(*mut MyPlugin),
    // 0x10 (Slot 4)
    pub get_plugin_description: extern "thiscall" fn(*mut MyPlugin) -> *const c_char,
    // 0x14 (Slot 5)
    pub level_init: extern "thiscall" fn(*mut MyPlugin, *const c_char),
    // 0x18 (Slot 6)
    pub server_activate: extern "thiscall" fn(*mut MyPlugin, *mut c_void, c_int, c_int),
    // 0x1C (Slot 7)
    pub game_frame: extern "thiscall" fn(*mut MyPlugin, bool),
    // 0x20 (Slot 8)
    pub level_shutdown: extern "thiscall" fn(*mut MyPlugin),
    // 0x24 (Slot 9)
    pub client_active: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    // 0x28 (Slot 10)
    pub client_disconnect: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    // 0x2C (Slot 11)
    pub client_put_in_server: extern "thiscall" fn(*mut MyPlugin, *mut c_void, *const c_char),
    // 0x30 (Slot 12)
    pub set_command_client: extern "thiscall" fn(*mut MyPlugin, c_int),
    // 0x34 (Slot 13)
    pub client_settings_changed: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    // 0x38 (Slot 14)
    pub client_connect: extern "thiscall" fn(
        *mut MyPlugin,
        *mut bool,
        *mut c_void,
        *const c_char,
        *const c_char,
        *mut c_char,
        c_int,
    ) -> c_int,
    // 0x3C (Slot 15)
    pub client_command: extern "thiscall" fn(*mut MyPlugin, *mut c_void, *const c_void) -> c_int,
    // 0x40 (Slot 16)
    pub network_id_validated:
        extern "thiscall" fn(*mut MyPlugin, *const c_char, *const c_char) -> c_int,
    // 0x44 (Slot 17)
    pub on_query_cvar_value_finished: extern "thiscall" fn(
        *mut MyPlugin,
        c_int,
        *mut c_void,
        c_int,
        *const c_char,
        *const c_char,
    ),
    // 0x48 (Slot 18)
    pub on_edict_allocated: extern "thiscall" fn(*mut MyPlugin, *mut c_void),
    // 0x4C (Slot 19)
    pub on_edict_freed: extern "thiscall" fn(*mut MyPlugin, *const c_void),
}

pub extern "thiscall" fn load(
    _this: *mut MyPlugin,
    interface_factory: *const c_void,
    _server_factory: *const c_void,
) -> bool {
    crate::vtable::load(interface_factory);
    true
}

pub extern "thiscall" fn level_init(_this: *mut MyPlugin, p_map_name: *const c_char) {
    let Ok(map_name) = unsafe { CStr::from_ptr(p_map_name) }.to_str() else {
        return;
    };
    crate::vtable::level_init(map_name);
}

pub extern "thiscall" fn get_plugin_description(_this: *mut MyPlugin) -> *const c_char {
    DESC.as_ptr() as *const c_char
}

pub extern "thiscall" fn game_frame(_this: *mut MyPlugin, _simulating: bool) {
    crate::vtable::game_frame();
}

pub extern "thiscall" fn dummy_void(_this: *mut MyPlugin) {}
pub extern "thiscall" fn dummy_void_ptr(_this: *mut MyPlugin, _a: *mut c_void) {}
pub extern "thiscall" fn dummy_void_3(_this: *mut MyPlugin, _a: *mut c_void, _b: c_int, _c: c_int) {
}

pub extern "thiscall" fn dummy_void_2(_this: *mut MyPlugin, _a: *mut c_void, _b: *const c_char) {}
pub extern "thiscall" fn dummy_void_int(_this: *mut MyPlugin, _a: c_int) {}

pub extern "thiscall" fn dummy_int_6(
    _this: *mut MyPlugin,
    _a: *mut bool,
    _b: *mut c_void,
    _c: *const c_char,
    _d: *const c_char,
    _e: *mut c_char,
    _f: c_int,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_3_ptr(
    _this: *mut MyPlugin,
    _a: *mut c_void,
    _b: *const c_void,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_2c(
    _this: *mut MyPlugin,
    _a: *const c_char,
    _b: *const c_char,
) -> c_int {
    0
}

pub extern "thiscall" fn dummy_void_int_3(
    _this: *mut MyPlugin,
    _a: c_int,
    _b: *mut c_void,
    _c: c_int,
    _d: *const c_char,
    _e: *const c_char,
) {
}

pub extern "thiscall" fn dummy_void_ptr_const(_this: *mut MyPlugin, _a: *const c_void) {}
