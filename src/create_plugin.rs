use std::{
    fs::{self, File},
    path::Path,
    time::SystemTime,
};

use crate::{
    bridge::{MyPlugin, console},
    consts::CRASH_FORMAT,
    log,
    os::format_local_time,
    panic_handler,
    statics::*,
};

fn manage_crash_reports() {
    let _ = fs::create_dir("./crash_reports");
    {
        let filename = format_local_time(CRASH_FORMAT);
        let path = format!("./crash_reports/{filename}.txt");
        let path = Path::new(&path);
        if File::create(path).is_err() {
            return;
        }
        let _ = CRASH_FILENAME.set(filename.to_string().into());
        panic_handler::setup_panic_logging(path.into());
    }
    let Ok(entries) = fs::read_dir("./crash_reports") else {
        return;
    };
    let mut entries = entries
        .filter_map(|x| x.ok())
        .filter(|x| x.file_name().to_str().is_some_and(|x| x.ends_with(".txt")))
        .collect::<Vec<_>>();
    entries.sort_by_key(|x| {
        x.metadata()
            .and_then(|x| x.created())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    for entry in entries.iter().take(entries.len().saturating_sub(5)) {
        let _ = fs::remove_file(entry.path());
    }
}

pub fn create_plugin(requested_name: &str) -> Option<*mut MyPlugin> {
    log!("Engine requested interface: {requested_name}");

    if !requested_name.starts_with("ISERVERPLUGINCALLBACKS") {
        return None;
    }

    manage_crash_reports();
    let _ = fs::create_dir("portal2/records");

    CON_COLOR_MSG_FN
        .set(console::get_con_color_msg().expect("Failed to find game console"))
        .unwrap();

    let instance = MyPlugin::new(&VTABLE);
    let ptr = PLUGIN_INSTANCE.get_or_init(|| instance) as *const MyPlugin as *mut MyPlugin;

    log!("Plugin instance pointer returned");
    Some(ptr)
}
