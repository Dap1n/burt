use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    commands::{
        run::{rec, sr_stop, state},
        tas::{
            self, IN_RETRY, PARSED_TAS, RAW_LOGGER, RETRY_COUNTER, SESSION_COUNTER, restore,
            restore_cvars,
        },
    },
    game::{
        cvars,
        hooks::{
            Subscriber, input_filter,
            render_view::{self, RenderViewArgs},
        },
        provider::provider,
    },
};

pub static IN_LOAD: AtomicBool = AtomicBool::new(false);

fn on_render(args: &mut RenderViewArgs) {
    if !state::in_speedrun() || rec::CURRENT_MAP.read().unwrap().as_str() != provider().last_map() {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        return;
    }
    let view = unsafe { render_view::read_setup_origin(args.setup as *const f32) };
    if provider().in_end_position(view) {
        render_view::unsubscribe(Subscriber::Speedrun(on_render));
        sr_stop::stop_internal(true);
    }
}

fn manage_tas() {
    if IN_RETRY.swap(false, Ordering::Relaxed) {
        return;
    }
    match SESSION_COUNTER.fetch_sub(1, Ordering::Relaxed) {
        1 => {
            // TAS finished via new session load
            input_filter::set_keyinputs_enabled(true);
            cvars::set_mouseenable(true);
            restore();
            restore_cvars();
            RETRY_COUNTER.store(0, Ordering::Relaxed);
            PARSED_TAS.write().unwrap().take();
            let mut logger_write = RAW_LOGGER.write().unwrap();
            logger_write.take();
        }
        2 => {
            // TAS should start
            input_filter::set_keyinputs_enabled(false);
            cvars::set_mouseenable(false);
            tas::restore();
            tas::set_cvars();
            RETRY_COUNTER.store(0, Ordering::Relaxed);
        }
        3 => (), // wait for the next session
        _ => unreachable!(),
    }
}

pub fn level_init(map_name: &str) {
    *rec::CURRENT_MAP.write().unwrap() = map_name.to_string();
    IN_LOAD.store(true, Ordering::Relaxed);
    if state::in_speedrun() && map_name == provider().last_map() {
        render_view::subscribe(Subscriber::Speedrun(on_render));
    }
    if SESSION_COUNTER.load(Ordering::Relaxed) != 0 {
        manage_tas();
    }
}
