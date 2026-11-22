use crate::register_commands;

mod absmov;
mod autoaim;
pub mod autojump;
pub mod bvars;
mod clear_mdmp;
mod decel;
mod echo;
mod exit;
mod getpos_exact;
pub mod j2009;
pub mod p2008;
mod portal_report;
pub mod run;
mod setang;
mod sg_dbg;
pub mod strafe;
pub mod tas;

register_commands! {
    absmov::burt_absmov => "Absolute movement toward a world angle",
    autoaim::burt_autoaim_entity => "Automatic aim at certain entity",
    autoaim::burt_autoaim_point => "Automatic aim at certain coordinates",
    autojump::burt_autojump => "Toggle automatic jumping when getting grounded",
    p2008::burt_capture_report => "Display information about currently captured object",
    clear_mdmp::burt_clear_mdmp => "Remove all .mdmp files",
    decel::burt_decel => "Stop as fast as possible via ground friction",
    echo::burt_echo => "Print colored text to the console",
    exit::burt_exit => "Unload the plugin",
    getpos_exact::burt_getpos_exact => "Get exact position and angles of the player entity",
    portal_report::burt_portal_report => "Get positions and angles of portals placed by player",
    run::rec::burt_rec => "Toggle automatic demo recording on level load",
    p2008::burt_scale => "Scale the captured object",
    setang::burt_setang => "Rotate camera to angles",
    sg_dbg::burt_sg_dbg => "Manage Save Glitch debugging",
    run::sr_start::burt_sr_start => "Start the speedrun",
    run::sr_stop::burt_sr_stop => "End the speedrun",
    strafe::j2009::burt_strafe => "TAS strafe tool",
    strafe::p2008::burt_strafe => "TAS strafe tool",
    tas::burt_tas_advance => "Advance TAS playback by the specified amount of ticks",
    tas::burt_pause => "Pause the game",
    tas::burt_tas_play | tas::burt_tas_play_autocomplete => "Play a TAS script with given name",
    tas::burt_tas_replay => "Replay the last played TAS",
    tas::burt_resume => "Resume the game",
    tas::burt_tas_stop => "Stop the current TAS",

    bvars::bvar_tas_check_disable => "Globally disable the TAS check tool",
    bvars::bvar_tas_check_max_replays => "How many times the check tool will restart playback",
    bvars::bvar_tas_commands_disable => "Globally disable commands in TAS script",
    bvars::bvar_tas_interpolate => "Preserve client interpolation in TAS playback",
    bvars::bvar_tas_pause_at => "Automatically pause the TAS at the given tick",
    bvars::bvar_tas_playback_rate => "The rate at which to play back TAS scripts",
    j2009::bvars::bvar_tas_quantize_movement => "Whether to quantize movement",
    bvars::bvar_tas_skip_to => "Quickly forward the TAS up to the given tick",
    bvars::bvar_tas_tools_disable => "Globally disable all TAS tools",
}
