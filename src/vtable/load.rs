use std::{mem, os::raw::c_void};

use crate::{
    bridge::{self, commands::CreateInterfaceFn, invoke::EngineServer},
    clog,
    game::hooks,
    log,
    statics::ENGINE,
    structs::Color,
};

type Successful = bool;

pub fn load(interface_factory: *const c_void) -> Successful {
    log!("Load called");

    if interface_factory.is_null() {
        clog!(err_prefix; "Interface factory is null! Aborting...");
        return false;
    }

    let factory: CreateInterfaceFn = unsafe { mem::transmute(interface_factory) };

    let Some(engine) = (unsafe { EngineServer::from_factory(factory) }) else {
        clog!(err_prefix; "Failed to retrieve EngineServer! Aborting...");
        return false;
    };

    let _ = ENGINE.set(engine);

    bridge::commands::register_plugin_commands(factory);
    hooks::install_all();

    clog!(prefix;
        [Color::WHITE =>
            "Some commands may run without arguments. \
            If you want to see their description,\n"],
        [Color::WHITE => "consider running them with a question mark: "],
        [Color::YELLOW => "burt_sr_start ?"],
    );

    true
}
