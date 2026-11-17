use burt_command::burt_command;

use crate::{
    clog,
    game::player::{self, CaptureReport},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleStep {
    Exact(i32),
    Relative(i32),
    Min,
    Max,
}

pub fn enable_scale(report: CaptureReport, step: ScaleStep) -> i32 {
    let scale = match step {
        ScaleStep::Exact(step) => {
            step.clamp(report.min_bound, report.max_bound) - report.current_step
        }
        ScaleStep::Relative(steps) if steps > 0 => {
            steps.min(report.max_bound - report.current_step)
        }
        ScaleStep::Relative(steps) => steps.max(report.min_bound - report.current_step),
        ScaleStep::Min => report.min_bound - report.current_step,
        ScaleStep::Max => report.max_bound - report.current_step,
    };
    player::execute_scaling(scale);
    scale
}

/// Scales currently captured object to the provided step.
///
/// `step:` target step:
///   - `min` — scale to the minimum possible size
///   - `max` — scale to the maximum possible size
///   - `up [N]` — scales up by N steps (1 by default). Example: `burt_scale up_2`
///   - `down [N]` — scales down by N steps (1 by default). Example: `burt_scale "down 3"`
///   - if integer is provided, scales to the exact step
#[burt_command]
pub fn burt_scale(step: ScaleStep) {
    let Some(report) = player::get_capture_report() else {
        clog!(err_prefix; "No objects captured!");
        return;
    };
    if !report.in_placement {
        clog!(err_prefix; "Not in object placement, ignoring");
        return;
    }
    enable_scale(report, step);
}
