use std::sync::OnceLock;

use crate::bridge::{BurtPlugin, console::ConColorMsgFn, invoke::EngineServer, vtable::*};

pub static CRASH_FILENAME: OnceLock<Box<str>> = OnceLock::new();
pub static PLUGIN_INSTANCE: OnceLock<BurtPlugin> = OnceLock::new();
pub static CON_COLOR_MSG_FN: OnceLock<ConColorMsgFn> = OnceLock::new();
pub static ENGINE: OnceLock<EngineServer> = OnceLock::new();

pub static ALIEN_SWARM_VTABLE: AlienSwarmVTable = AlienSwarmVTable {
    load,
    unload,
    pause: dummy_void,
    un_pause: dummy_void,
    get_plugin_description,
    level_init,
    server_activate: dummy_void_3,
    game_frame,
    level_shutdown: dummy_void,
    client_active: dummy_void_ptr,
    client_fully_connect: dummy_void_ptr,
    client_disconnect: dummy_void_ptr,
    client_put_in_server: dummy_void_2,
    set_command_client: dummy_void_int,
    client_settings_changed: dummy_void_ptr,
    client_connect: dummy_int_6,
    client_command: dummy_void_3_ptr,
    network_id_validated: dummy_void_2c,
    on_query_cvar_value_finished: dummy_void_int_3,
    on_edict_allocated: dummy_void_ptr,
    on_edict_freed: dummy_void_ptr_const,
};
pub static JULY_2009_VTABLE: July2009VTable = July2009VTable {
    load,
    unload,
    pause: dummy_void,
    un_pause: dummy_void,
    get_plugin_description,
    level_init,
    server_activate: dummy_void_3,
    game_frame,
    level_shutdown: dummy_void,
    client_active: dummy_void_ptr,
    client_disconnect: dummy_void_ptr,
    client_put_in_server: dummy_void_2,
    set_command_client: dummy_void_int,
    client_settings_changed: dummy_void_ptr,
    client_connect: dummy_int_6,
    client_command: dummy_void_3_ptr,
    network_id_validated: dummy_void_2c,
    on_query_cvar_value_finished: dummy_void_int_3,
    on_edict_allocated: dummy_void_ptr,
    on_edict_freed: dummy_void_ptr_const,
};
