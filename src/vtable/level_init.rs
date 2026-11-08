use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    commands::{
        run::{rec, sr_stop, state},
        tas,
    },
    engine_build::{self, EngineBuild},
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
    let build = engine_build::current_build();
    let mut check = if build == EngineBuild::Project2008 {
        rec::CURRENT_MAP.read().unwrap().as_str() != "lab_fan_lift"
    } else {
        rec::CURRENT_MAP.read().unwrap().as_str() != "p2_lab_hub_6"
    };
    if !state::in_speedrun() || check {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        return;
    }
    let view = unsafe { render_view::read_setup_origin(args.setup as *const f32) };
    check = if build == EngineBuild::Project2008 {
        (-2980.0..=-2840.0).contains(&view.0)
            && (-2310.0..=-2280.0).contains(&view.1)
            && (-3072.0..=-2950.0).contains(&view.2)
    } else {
        view == (4096.0, 3220.0, 4096.0)
    };
    if check {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        sr_stop::stop_internal(true);
    }
}

pub fn level_init(map_name: &str) {
    *rec::CURRENT_MAP.write().unwrap() = map_name.to_string();
    IN_LOAD.store(true, Ordering::Relaxed);
    let check = if engine_build::current_build() == EngineBuild::Project2008 {
        map_name == "lab_fan_lift"
    } else {
        map_name == "p2_lab_hub_6"
    };
    if state::in_speedrun() && check {
        render_view::subscribe(Subscriber::Speedrun(on_render));
    }
    if tas::SESSION_COUNTER.load(Ordering::Relaxed) == 2 {
        input_filter::set_keyinputs_enabled(false);
        cvars::set_mouseenable(false);
        tas::LAST_JOYSTICK.store(cvars::JOYSTICK.int_or(1) != 0, Ordering::Relaxed);
        cvars::set_joystick(false);
    }
}
