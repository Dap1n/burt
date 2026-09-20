use std::sync::OnceLock;

use crate::bridge::{MyPlugin, console::ConColorMsgFn, invoke::EngineServer, vtable::*};

pub static DESC: &[u8] = b"BURT (Beta Utility for Recording & Tracing) v0.1\0";

pub static CRASH_FILENAME: OnceLock<Box<str>> = OnceLock::new();
pub static PLUGIN_INSTANCE: OnceLock<MyPlugin> = OnceLock::new();
pub static CON_COLOR_MSG_FN: OnceLock<ConColorMsgFn> = OnceLock::new();
pub static ENGINE: OnceLock<EngineServer> = OnceLock::new();

pub static VTABLE: PluginVTable = PluginVTable {
    // 0x00 (Slot 0)
    load,
    // 0x04 (Slot 1)
    unload: dummy_void,
    // 0x08 (Slot 2)
    pause: dummy_void,
    // 0x0C (Slot 3)
    un_pause: dummy_void,
    // 0x10 (Slot 4)
    get_plugin_description,
    // 0x14 (Slot 5)
    level_init,
    // 0x18 (Slot 6)
    server_activate: dummy_void_3,
    // 0x1C (Slot 7)
    game_frame,
    // 0x20 (Slot 8)
    level_shutdown: dummy_void,
    // 0x24 (Slot 9)
    client_active: dummy_void_ptr,
    // 0x28 (Slot 10)
    client_disconnect: dummy_void_ptr,
    // 0x2C (Slot 11)
    client_put_in_server: dummy_void_2,
    // 0x30 (Slot 12)
    set_command_client: dummy_void_int,
    // 0x34 (Slot 13)
    client_settings_changed: dummy_void_ptr,
    // 0x38 (Slot 14)
    client_connect: dummy_int_6,
    // 0x3C (Slot 15)
    client_command: dummy_void_3_ptr,
    // 0x40 (Slot 16)
    network_id_validated: dummy_void_2c,
    // 0x44 (Slot 17)
    on_query_cvar_value_finished: dummy_void_int_3,
    // 0x48 (Slot 18)
    on_edict_allocated: dummy_void_ptr,
    // 0x4C (Slot 19)
    on_edict_freed: dummy_void_ptr_const,
};
