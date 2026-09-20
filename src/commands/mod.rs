use crate::register_commands;

mod echo;
pub mod rec;
pub mod run_state;
mod sg_dbg;
pub mod sr_start;
pub mod sr_stop;

register_commands! {
    "burt_echo" => "Custom echo command" => echo::burt_echo,
    "burt_sg_dbg" => "Manages Save Glitch debugging" => sg_dbg::burt_sg_dbg,
    "burt_rec" => "Toggle automatic demo recording on level load" => rec::burt_rec,
    "burt_sr_start" => "Start the speedrun" => sr_start::burt_sr_start,
    "burt_sr_stop" => "End the speedrun" => sr_stop::burt_sr_stop,
}
