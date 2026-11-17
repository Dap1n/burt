use burt_command::burt_command;

use crate::{
    clog,
    game::{entity, player::get_capture_report},
    structs::Color,
};

/// Displays information about the currently captured object.
///
/// The command prints out:
/// - Entity name and classname
/// - Current resizing step and multiplier
/// - Step bounds
/// - "Inactive" if you're not currently trying to place the object
#[burt_command]
pub fn burt_capture_report() {
    let Some(report) = get_capture_report() else {
        clog!(err_prefix; "No objects captured!");
        return;
    };
    let name =
        unsafe { entity::entity_name(report.entity).unwrap_or_else(|| "<none>".to_string()) };
    let classname =
        unsafe { entity::entity_classname(report.entity).unwrap_or_else(|| "<none>".to_string()) };
    clog!(
        [Color::YELLOW => "Captured: "],
        [Color::WHITE => "{name} ({classname})\n"],
        [Color::YELLOW => "Step: "],
        [Color::WHITE => "{} ({}x)\n", report.current_step, report.multiplier],
        [Color::YELLOW => "Bounds: "],
        [Color::WHITE => "[{}, {}]", report.min_bound, report.max_bound],
        [Color::GRAY => "{}", if report.in_placement { "" } else { "\nInactive" }],
    );
}
