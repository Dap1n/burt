use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    commands::{
        run::{rec, sr_stop, state},
        tas,
    },
    consts::LAST_MAP_NAME,
    game::{
        cvars,
        hooks::{
            Subscriber, input_filter,
            render_view::{self, RenderViewArgs},
        },
    },
};

pub static IN_LOAD: AtomicBool = AtomicBool::new(false);

fn on_render(args: &mut RenderViewArgs) {
    if !state::in_speedrun() || rec::CURRENT_MAP.read().unwrap().as_str() != LAST_MAP_NAME {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        return;
    }
    let view = unsafe { render_view::read_setup_origin(args.setup as *const f32) };
    if view == (4096.0, 3220.0, 4096.0) {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        sr_stop::stop_internal(true);
    }
}

pub fn level_init(map_name: &str) {
    *rec::CURRENT_MAP.write().unwrap() = map_name.to_string();
    IN_LOAD.store(true, Ordering::Relaxed);
    if state::in_speedrun() && map_name == LAST_MAP_NAME {
        render_view::subscribe(Subscriber::Speedrun(on_render));
    }
    if tas::SESSION_COUNTER.load(Ordering::Relaxed) == 2 {
        input_filter::set_keyinputs_enabled(false);
        cvars::set_mouseenable(false);
        tas::LAST_JOYSTICK.store(cvars::JOYSTICK.int_or(1) != 0, Ordering::Relaxed);
        cvars::set_joystick(false);
    }
}
