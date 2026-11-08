use std::{
    fs::{self, File},
    path::Path,
    time::SystemTime,
};

use crate::{
    bridge::{BurtPlugin, console},
    clog,
    consts::CRASH_FORMAT,
    engine_build::{self, EngineBuild},
    game::{modules, offsets_provider},
    log,
    os::format_local_time,
    panic_handler,
    statics::*,
    structs::Color,
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
    entries
        .iter()
        .take(entries.len().saturating_sub(5))
        .for_each(|entry| {
            let _ = fs::remove_file(entry.path());
        });
}

pub fn create_plugin(requested_name: &str) -> Option<*mut BurtPlugin> {
    if requested_name != "ISERVERPLUGINCALLBACKS003" {
        return None;
    }

    let _ = CON_COLOR_MSG_FN
        .get_or_init(|| console::get_con_color_msg().expect("Failed to find game console"));

    modules::init();

    let Some(build) = EngineBuild::get_current() else {
        clog!(err_prefix; "Unknown build! Aborting...");
        return None;
    };
    clog!(prefix;
        [Color::WHITE => "Build: "],
        [Color::YELLOW => "{} ({})", build.name(), build.tag()],
    );
    if !offsets_provider::init_active_build(build) {
        clog!(err_prefix; "This build is not supported yet");
        return None;
    }
    engine_build::save(build);

    manage_crash_reports();

    let instance = if build == EngineBuild::Project2008 {
        let _ = fs::create_dir("p2008/records");
        BurtPlugin::new(&ALIEN_SWARM_VTABLE)
    } else {
        let _ = fs::create_dir("portal2/records");
        BurtPlugin::new(&JULY_2009_VTABLE)
    };

    let ptr = PLUGIN_INSTANCE.get_or_init(|| instance) as *const BurtPlugin as *mut BurtPlugin;
    log!("Plugin instance pointer returned");
    Some(ptr)
}
