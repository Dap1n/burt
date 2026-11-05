use std::{mem, os::raw::c_void};

use crate::{
    bridge::{self, commands::CreateInterfaceFn, invoke::EngineServer},
    clog,
    engine_build::EngineBuild,
    game::{hooks, modules},
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

    modules::init();

    if let Some(build) = EngineBuild::get_current() {
        clog!(prefix;
            [Color::WHITE => "Build: "],
            [Color::YELLOW => "{} ({})", build.name(), build.number()],
        );
        if build != EngineBuild::July2009 {
            clog!(err_prefix; "This build is not supported yet");
            return false;
        }
    } else {
        clog!(err_prefix; "Unknown build! Aborting...");
        return false;
    }

    let factory: CreateInterfaceFn = unsafe { mem::transmute(interface_factory) };

    let Some(engine) = (unsafe { EngineServer::from_factory(factory) }) else {
        clog!(err_prefix; "Failed to retrieve EngineServer! Aborting...");
        return false;
    };

    ENGINE.set(engine).unwrap();

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
