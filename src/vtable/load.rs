use std::{mem, os::raw::c_void};

use crate::{
    bridge::{self, commands::CreateInterfaceFn, console::Color, invoke::EngineServer},
    clog, log,
    statics::ENGINE,
};

pub fn load(interface_factory: *const c_void) {
    log!("Load called");
    clog!(prefix; "Plugin is loading...");

    if interface_factory.is_null() {
        clog!(prefix; "Interface factory is null! Commands will be unavailable");
        return;
    }

    let factory: CreateInterfaceFn = unsafe { mem::transmute(interface_factory) };
    bridge::commands::register_plugin_commands(factory);
    clog!(prefix; "Commands registered");

    clog!(prefix;
        [Color::WHITE => "Some commands may run without arguments. If you want \
        to see their description, consider running them with a question mark: "],
        [Color::LIME => "burt_sr_start ?"],
    );

    if let Some(engine) = unsafe { EngineServer::from_factory(factory) } {
        ENGINE.set(engine).unwrap();
    }
}
