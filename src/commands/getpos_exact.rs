use burt_command::burt_command;

use crate::{
    clog,
    game::{hooks::process_cmd, player},
    structs::Color,
};

/// Gets exact position and angles of the player entity.
///
/// During Save Glitch player's camera and body may differ (if you behind the portal
/// you performed SG on, for instance), so this command will show position
/// and angles of the body. It will also skip `roll` parameter in angles,
/// since it's constantly `0.0` in this context.
#[burt_command]
pub fn burt_getpos_exact() {
    let Some(origin) = player::origin_pos() else {
        clog!(err_prefix; "Can't find player entity");
        return;
    };
    let angles = process_cmd::cached_body_angles();
    clog!(
        [Color::YELLOW => "Origin: "],
        [Color::WHITE => "{:.6} {:.6} {:.6}\n", origin.0, origin.1, origin.2],
        [Color::YELLOW => "Angles: "],
        [Color::WHITE => "{:.6} {:.6}", angles.0, angles.1],
    );
}
