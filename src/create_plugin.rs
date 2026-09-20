use std::fs::{self, File};

use chrono::Local;

use crate::{
    bridge::{MyPlugin, console},
    consts::CRASH_FORMAT,
    log,
    statics::{CON_COLOR_MSG_FN, CRASH_FILENAME, PLUGIN_INSTANCE, VTABLE},
};

pub fn create_plugin(requested_name: &str) -> Option<*mut MyPlugin> {
    log!("Engine requested interface: {requested_name}");

    if !requested_name.starts_with("ISERVERPLUGINCALLBACKS") {
        return None;
    }

    let _ = fs::create_dir("crash_reports");
    let _ = fs::create_dir("portal2/records");
    let filename = Local::now().format(CRASH_FORMAT);
    let _ = File::create(format!("crash_reports/{filename}.txt"));
    CRASH_FILENAME.set(filename.to_string().into()).unwrap();

    CON_COLOR_MSG_FN
        .set(console::get_con_color_msg().expect("Failed to find game console"))
        .unwrap();

    let instance = MyPlugin::new(&VTABLE);
    let ptr = PLUGIN_INSTANCE.get_or_init(|| instance) as *const MyPlugin as *mut MyPlugin;

    log!("Plugin instance pointer returned");
    Some(ptr)
}
