use std::{
    fs,
    os::raw::c_void,
    path::Path,
    sync::{
        RwLock,
        atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
};

use portable_atomic::AtomicF32;

use crate::{
    bridge::commands::{AutocompleteError, AutocompleteWriter},
    cexec, clog,
    commands::{
        absmov, autoaim, autojump,
        bvars::*,
        decel,
        p2008::scale,
        setang,
        strafe::{self, math::length3d},
    },
    engine_build::{self, EngineBuild},
    game::{
        cinput::HELD_BUTTONS,
        cvars::{self, *},
        entity,
        hooks::{
            accumulate_time::{TIME_ADVANCE_TICKS, TIME_PAUSED},
            create_move::{self, CreateMoveArgs},
            input_filter,
            process_cmd::{self, schedule_movement},
        },
        player,
        provider::provider,
    },
    statics::ENGINE,
    structs::{Color, Coordinates, TargetEntity},
    vtable::IN_LOAD,
};
use burt_command::{burt_autocomplete, burt_command};

mod parser;
mod raw;

pub use parser::*;
pub use raw::{RAW_LOGGER, RawLogger};

pub static TICK: AtomicU32 = AtomicU32::new(0);
pub static SKIP: AtomicUsize = AtomicUsize::new(0);
pub static CURRENT_MOVEMENT: AtomicU64 = AtomicU64::new(0);
pub static CURRENT_ANGLES: AtomicU64 = AtomicU64::new(0);
pub static CURRENT_DX: AtomicF32 = AtomicF32::new(0.0);
pub static CURRENT_DY: AtomicF32 = AtomicF32::new(0.0);
pub static CURRENT_COMMANDS: RwLock<Option<Box<str>>> = RwLock::new(None);
pub static SESSION_COUNTER: AtomicU8 = AtomicU8::new(0);
pub static RETRY_COUNTER: AtomicU32 = AtomicU32::new(0);
pub static PARSED_TAS: RwLock<Option<TasScript>> = RwLock::new(None);

pub static LAST_FPS_MAX: AtomicF32 = AtomicF32::new(60.0);
pub static LAST_HOST_FRAMERATE: AtomicF32 = AtomicF32::new(0.0);
pub static LAST_SENSITIVITY: AtomicF32 = AtomicF32::new(3.0);
pub static LAST_M_YAW: AtomicF32 = AtomicF32::new(0.022);
pub static LAST_M_PITCH: AtomicF32 = AtomicF32::new(0.022);
pub static LAST_CL_PITCHUP: AtomicF32 = AtomicF32::new(89.0);
pub static LAST_CL_PITCHDOWN: AtomicF32 = AtomicF32::new(89.0);
pub static LAST_CL_FORWARDSPEED: AtomicF32 = AtomicF32::new(450.0);
pub static LAST_CL_SIDESPEED: AtomicF32 = AtomicF32::new(450.0);
pub static LAST_CL_BACKSPEED: AtomicF32 = AtomicF32::new(450.0);
pub static LAST_MAT_MOTION_BLUR: AtomicBool = AtomicBool::new(true);
pub static LAST_JOYSTICK: AtomicBool = AtomicBool::new(true);

pub static LAST_PLAYED_TAS: RwLock<Option<Box<str>>> = RwLock::new(None);

fn is_save_exists(name: &str) -> bool {
    let Ok(read_dir) = fs::read_dir(format!("./{}/SAVE", provider().main_folder())) else {
        return false;
    };
    read_dir
        .filter_map(|x| x.ok())
        .any(|x| x.file_name().to_str().and_then(|x| x.strip_suffix(".sav")) == Some(name))
}

fn is_map_exists(name: &str) -> bool {
    let Ok(read_dir) = fs::read_dir(format!("./{}/maps", provider().main_folder())) else {
        return false;
    };
    read_dir
        .filter_map(|x| x.ok())
        .any(|x| x.file_name().to_str().and_then(|x| x.strip_suffix(".bsp")) == Some(name))
}

fn disable_all_tools() {
    setang::disable_setang();
    autoaim::disable_autoaim();
    autojump::disable_autojump();
    absmov::disable_absmov();
    if engine_build::current_build() == EngineBuild::Project2008 {
        strafe::p2008::disable_strafe();
    } else {
        strafe::j2009::disable_strafe();
    }
    decel::disable_decel();
}

fn release_all_buttons() {
    HeldButtons(HELD_BUTTONS.load(Ordering::Relaxed))
        .parse_actions()
        .for_each(|action| action.release(0));
}

pub fn restore() {
    TICK.store(0, Ordering::Relaxed);
    SKIP.store(0, Ordering::Relaxed);
    CURRENT_MOVEMENT.store(0, Ordering::Relaxed);
    CURRENT_ANGLES.store(0, Ordering::Relaxed);
    CURRENT_DX.store(0.0, Ordering::Relaxed);
    CURRENT_DY.store(0.0, Ordering::Relaxed);
    CURRENT_COMMANDS.write().unwrap().take();

    disable_all_tools();
    release_all_buttons();
}

fn handle_tool_change(change: &ToolChange) {
    match change {
        ToolChange::Enable(Tool::Check { .. }) => (), // already handled
        ToolChange::Enable(Tool::Scale { .. }) => (), // already handled
        &ToolChange::Enable(Tool::Setang {
            pitch,
            yaw,
            time,
            easing,
        }) => {
            setang::enable_setang(pitch, yaw, time, easing);
        }
        &ToolChange::Enable(Tool::Autoaim {
            x,
            y,
            z,
            time,
            easing,
        }) => {
            autoaim::enable_autoaim(Coordinates { x, y, z }, time, easing);
        }
        ToolChange::Enable(Tool::AutoaimEnt {
            name,
            index,
            offset_x,
            offset_y,
            offset_z,
            time,
            easing,
        }) => {
            let entity = TargetEntity {
                name: name.clone(),
                index: *index,
            };
            let Some((edict, ent)) = entity.fetch() else {
                if let Some(name) = name.as_deref() {
                    clog!(err_prefix;
                        [Color::WHITE => "Can't find entity "],
                        [Color::RED => "{name}[{index}]"],
                    );
                } else {
                    clog!(err_prefix;
                        [Color::WHITE => "Can't find entity "],
                        [Color::RED => "{index}"],
                    );
                }
                return;
            };
            autoaim::enable_autoaim_entity(
                edict,
                ent,
                Coordinates {
                    x: *offset_x,
                    y: *offset_y,
                    z: *offset_z,
                },
                *time,
                *easing,
            );
        }
        &ToolChange::Enable(Tool::Autojump { ducked }) => {
            autojump::enable_autojump(ducked);
        }
        &ToolChange::Enable(Tool::Absmov { angle, strength }) => {
            absmov::enable_absmov(angle, strength);
        }
        &ToolChange::Enable(Tool::ModernStrafe { params }) => {
            strafe::j2009::enable_strafe(params);
        }
        &ToolChange::Enable(Tool::LegacyStrafe { params }) => {
            strafe::p2008::enable_strafe(params);
        }
        &ToolChange::Enable(Tool::Decel { speed }) => {
            decel::enable_decel(speed);
        }
        ToolChange::Disable(ToolKind::Autoaim) => {
            autoaim::disable_autoaim();
        }
        ToolChange::Disable(ToolKind::Autojump) => {
            autojump::disable_autojump();
        }
        ToolChange::Disable(ToolKind::Absmov) => {
            absmov::disable_absmov();
        }
        ToolChange::Disable(ToolKind::Strafe) => {
            if engine_build::current_build() == EngineBuild::Project2008 {
                strafe::p2008::disable_strafe();
            } else {
                strafe::j2009::disable_strafe();
            }
        }
        ToolChange::Disable(ToolKind::Decel) => {
            decel::disable_decel();
        }
    }
}

fn check_passed(params: &CheckParams) -> bool {
    if let Some((current_x, current_y, current_z)) = player::origin_pos()
        && let Some([x, y, z]) = params.pos
        && (!(x - params.posepsilon..=x + params.posepsilon).contains(&current_x)
            || !(y - params.posepsilon..=y + params.posepsilon).contains(&current_y)
            || !(z - params.posepsilon..=z + params.posepsilon).contains(&current_z))
    {
        clog!(err_prefix;
            [Color::WHITE => "Expected position to be "],
            [Color::RED => "{x} {y} {z}"],
            [Color::WHITE => " (posepsilon "],
            [Color::RED => "{}", params.posepsilon],
            [Color::WHITE => "), found "],
            [Color::RED => "{current_x} {current_y} {current_z}"],
        );
        return false;
    }
    let (current_pitch, current_yaw) = process_cmd::cached_body_angles();
    if let Some([pitch, yaw]) = params.ang
        && (!(pitch - params.angepsilon..=pitch + params.angepsilon).contains(&current_pitch)
            || !(yaw - params.angepsilon..=yaw + params.angepsilon).contains(&current_yaw))
    {
        clog!(err_prefix;
            [Color::WHITE => "Expected angles to be "],
            [Color::RED => "{pitch} {yaw}"],
            [Color::WHITE => " (angepsilon "],
            [Color::RED => "{}", params.angepsilon],
            [Color::WHITE => "), found "],
            [Color::RED => "{current_pitch} {current_yaw}"],
        );
        return false;
    }
    if let Some(current_velocity) = player::velocity().map(length3d)
        && let Some(velocity) = params.vel
        && !(velocity - params.velepsilon..=velocity + params.velepsilon)
            .contains(&current_velocity)
    {
        clog!(err_prefix;
            [Color::WHITE => "Expected velocity to be "],
            [Color::RED => "{velocity}"],
            [Color::WHITE => " (velepsilon "],
            [Color::RED => "{}", params.velepsilon],
            [Color::WHITE => "), found "],
            [Color::RED => "{current_velocity}"],
        );
        return false;
    }
    let holding_check_pass = match params.holding {
        Holding::NotChecked => true,
        Holding::Anything => player::is_holding(),
        Holding::Named(ref name) => {
            if let Some(ent) = unsafe { player::get_held_object() } {
                let ent_name = unsafe { entity::entity_name(ent) };
                let ent_classname = unsafe { entity::entity_classname(ent) };
                ent_name.as_deref() == Some(name) || ent_classname.as_deref() == Some(name)
            } else {
                false
            }
        }
    };
    if !holding_check_pass {
        clog!(err_prefix; "Holding check failed!");
    }
    holding_check_pass
}

pub fn set_cvars() {
    LAST_HOST_FRAMERATE.store(HOST_FRAMERATE.float_or(0.0), Ordering::Relaxed);
    HOST_FRAMERATE.set_float(provider().tickrate());

    LAST_SENSITIVITY.store(SENSITIVITY.float_or(3.0), Ordering::Relaxed);
    SENSITIVITY.set_float(3.0);
    LAST_M_YAW.store(M_YAW.float_or(0.022), Ordering::Relaxed);
    M_YAW.set_float(0.022);
    LAST_M_PITCH.store(M_PITCH.float_or(0.022), Ordering::Relaxed);
    M_PITCH.set_float(0.022);

    LAST_CL_PITCHUP.store(CL_PITCHUP.float_or(89.0), Ordering::Relaxed);
    CL_PITCHUP.set_float(89.0);
    LAST_CL_PITCHDOWN.store(CL_PITCHDOWN.float_or(89.0), Ordering::Relaxed);
    CL_PITCHDOWN.set_float(89.0);

    LAST_CL_FORWARDSPEED.store(CL_FORWARDSPEED.float_or(450.0), Ordering::Relaxed);
    CL_FORWARDSPEED.set_float(450.0);
    LAST_CL_SIDESPEED.store(CL_SIDESPEED.float_or(450.0), Ordering::Relaxed);
    CL_SIDESPEED.set_float(450.0);
    LAST_CL_BACKSPEED.store(CL_BACKSPEED.float_or(450.0), Ordering::Relaxed);
    CL_BACKSPEED.set_float(450.0);

    cvars::set_cl_interpolate(BVAR_TAS_INTERPOLATE.load(Ordering::Relaxed));
    LAST_MAT_MOTION_BLUR.store(MAT_MOTION_BLUR_ENABLED.int_or(1) != 0, Ordering::Relaxed);
    cvars::set_mat_motion_blur(false);

    LAST_JOYSTICK.store(cvars::JOYSTICK.int_or(1) != 0, Ordering::Relaxed);
    cvars::set_joystick(false);
}

pub fn restore_cvars() {
    FPS_MAX.set_float(LAST_FPS_MAX.load(Ordering::Relaxed));
    HOST_FRAMERATE.set_float(LAST_HOST_FRAMERATE.load(Ordering::Relaxed));

    SENSITIVITY.set_float(LAST_SENSITIVITY.load(Ordering::Relaxed));
    M_YAW.set_float(LAST_M_YAW.load(Ordering::Relaxed));
    M_PITCH.set_float(LAST_M_PITCH.load(Ordering::Relaxed));

    CL_PITCHUP.set_float(LAST_CL_PITCHUP.load(Ordering::Relaxed));
    CL_PITCHDOWN.set_float(LAST_CL_PITCHDOWN.load(Ordering::Relaxed));

    CL_FORWARDSPEED.set_float(LAST_CL_FORWARDSPEED.load(Ordering::Relaxed));
    CL_SIDESPEED.set_float(LAST_CL_SIDESPEED.load(Ordering::Relaxed));
    CL_BACKSPEED.set_float(LAST_CL_BACKSPEED.load(Ordering::Relaxed));

    cvars::set_cl_interpolate(true);
    cvars::set_mat_motion_blur(LAST_MAT_MOTION_BLUR.load(Ordering::Relaxed));
    cvars::set_joystick(LAST_JOYSTICK.load(Ordering::Relaxed));
}

pub fn end_tas() {
    input_filter::set_keyinputs_enabled(true);
    cvars::set_mouseenable(true);
    restore();
    restore_cvars();
    SESSION_COUNTER.store(0, Ordering::Relaxed);
    PARSED_TAS.write().unwrap().take();
    let mut logger_write = RAW_LOGGER.write().unwrap();
    logger_write.take();
}

#[inline]
pub fn tas_driving() -> bool {
    SESSION_COUNTER.load(Ordering::Relaxed) == 1
        && !IN_LOAD.load(Ordering::Relaxed)
        && !(engine_build::current_build() == EngineBuild::Project2008
            && TIME_PAUSED.load(Ordering::Relaxed)
            && TIME_ADVANCE_TICKS.load(Ordering::Relaxed) == 0)
}

pub fn on_create_move(args: &mut CreateMoveArgs) {
    if !tas_driving() {
        return;
    }

    CURRENT_DX.store(0.0, Ordering::Relaxed);
    CURRENT_DY.store(0.0, Ordering::Relaxed);

    unsafe {
        let mouse_active_ptr = (args.this as *mut u8).add(provider().cinput_mouse_active_off());
        if *mouse_active_ptr == 0 {
            let vtable = *(args.this as *const *const usize);

            // Call ActivateMouse() so we can change angles immediately
            let activate_mouse: unsafe extern "thiscall" fn(*mut c_void) =
                std::mem::transmute(*vtable.add(provider().cinput_activate_mouse_slot()));

            activate_mouse(args.this);
        }
    }

    let tick = TICK.fetch_add(1, Ordering::Relaxed);

    if tick == 0 {
        LAST_FPS_MAX.store(FPS_MAX.float_or(60.0), Ordering::Relaxed);
        if BVAR_TAS_SKIP_TO.load(Ordering::Relaxed) == 0 {
            FPS_MAX
                .set_float(provider().tickrate() * BVAR_TAS_PLAYBACK_RATE.load(Ordering::Relaxed));
        } else {
            FPS_MAX.set_int(0);
        }
    }

    let mut tas_finished = false;
    let mut gave_up = false;

    'tas_block: {
        let Some(parsed_tas_read) = PARSED_TAS.read().ok() else {
            return;
        };
        let Some(parsed_tas) = parsed_tas_read.as_ref() else {
            return;
        };

        let Some(intent) = parsed_tas.get_intent(tick, &SKIP, &CURRENT_MOVEMENT, &CURRENT_ANGLES)
        else {
            tas_finished = true;
            break 'tas_block;
        };

        let tools_disable = BVAR_TAS_TOOLS_DISABLE.load(Ordering::Relaxed);

        if !BVAR_TAS_CHECK_DISABLE.load(Ordering::Relaxed)
            && !tools_disable
            && let Some(tools_changes) = intent.tools
            && let Some(check_params) = tools_changes.iter().find_map(|t| {
                if let ToolChange::Enable(Tool::Check { params }) = t {
                    Some(params)
                } else {
                    None
                }
            })
            && !check_passed(check_params)
        {
            // TAS check tool assertion failed
            let retry = RETRY_COUNTER.fetch_add(1, Ordering::Relaxed);
            let max_replays = BVAR_TAS_CHECK_MAX_REPLAYS.load(Ordering::Relaxed);
            if max_replays != 0 && retry >= max_replays {
                clog!(err_prefix; "Too many retries. Giving up");
                gave_up = true;
                break 'tas_block;
            } else {
                clog!(err_prefix;
                    [Color::WHITE => "Retry "],
                    [Color::RED => "{}", retry + 1],
                    [Color::WHITE => ". Restarting..."],
                );
            }
            restore();
            'b: {
                let curr_tas_read = LAST_PLAYED_TAS.read().unwrap();
                let Some(curr_tas) = curr_tas_read.as_deref() else {
                    break 'b;
                };
                let extension = provider().tas_extension();
                let raw_path =
                    curr_tas[..curr_tas.len() - extension.len()].to_string() + "_raw" + extension;
                let Ok(logger) = RawLogger::new(raw_path, parsed_tas) else {
                    break 'b;
                };
                *RAW_LOGGER.write().unwrap() = Some(logger);
            }
            exec_tas_start(&parsed_tas.start);
            return;
        }
        if let Some(angles) = intent.angles {
            let (current_pitch, current_yaw) = process_cmd::cached_body_angles();
            let (pitch, yaw) = (current_pitch - angles.y, current_yaw - angles.x);
            create_move::schedule_angles(pitch, yaw);
        }
        if let Some(actions_press) = intent.actions_press {
            actions_press.for_each(|action| action.press(0));
        }
        if let Some(actions_release) = intent.actions_release {
            actions_release.for_each(|action| action.release(0));
        }
        let scale = if let Some(tools_changes) = intent.tools
            && let Some(step) = tools_changes.iter().find_map(|t| {
                if let ToolChange::Enable(Tool::Scale { step }) = t {
                    Some(step)
                } else {
                    None
                }
            })
            && let Some(report) = player::get_capture_report()
            && report.in_placement
        {
            scale::enable_scale(report, *step)
        } else {
            0
        };
        if !BVAR_TAS_COMMANDS_DISABLE.load(Ordering::Relaxed)
            && let Some(commands) = intent.commands
            && let Some(engine) = ENGINE.get()
        {
            engine.execute(commands, true);
            let commands = if scale != 0 {
                let s = if scale > 0 {
                    "mwheel_down;"
                } else {
                    "mwheel_up;"
                };
                (s.repeat(scale.unsigned_abs() as usize) + commands).into()
            } else {
                commands.into()
            };
            CURRENT_COMMANDS.write().unwrap().replace(commands);
        } else if scale != 0 {
            let s = if scale > 0 {
                "mwheel_down;"
            } else {
                "mwheel_up;"
            };
            CURRENT_COMMANDS.write().unwrap().replace(
                s.repeat(scale.unsigned_abs() as usize)
                    .trim_end_matches(';')
                    .into(),
            );
        }
        if !tools_disable && let Some(tools_changes) = intent.tools {
            tools_changes.iter().for_each(handle_tool_change);
        }
    }

    if tas_finished || gave_up {
        end_tas();
    }

    if tas_finished {
        clog!(prefix;
            [Color::WHITE => "TAS finished after "],
            [Color::YELLOW => "{tick} ticks"],
        );
    }
}

pub fn on_process_cmd() {
    if !tas_driving() {
        return;
    }
    let movement = Point2D::from_bits(CURRENT_MOVEMENT.load(Ordering::Relaxed));
    schedule_movement(movement.x, movement.y);
}

pub fn on_process_cmd_end(movement: Point2D) {
    if !tas_driving() {
        return;
    }
    let mut logger_write = RAW_LOGGER.write().unwrap();
    let Some(logger) = logger_write.as_mut() else {
        return;
    };
    let tick = TICK.load(Ordering::Relaxed) - 1;
    let angles = Point2D {
        x: CURRENT_DX.load(Ordering::Relaxed),
        y: CURRENT_DY.load(Ordering::Relaxed),
    };
    let buttons = HeldButtons(HELD_BUTTONS.load(Ordering::Relaxed));
    let commands = CURRENT_COMMANDS.write().unwrap().take();
    let _ = logger.tick(tick, movement, angles, buttons, commands);

    let pause_tick = BVAR_TAS_PAUSE_AT.load(Ordering::Relaxed);
    if pause_tick != 0 && tick == pause_tick {
        TIME_PAUSED.store(true, Ordering::Relaxed);
        if engine_build::current_build() == EngineBuild::Project2008 {
            HOST_FRAMERATE.set_float(provider().tickrate() + 0.0001);
        }
    }
    let skip_to = BVAR_TAS_SKIP_TO.load(Ordering::Relaxed);
    if skip_to != 0 && TICK.load(Ordering::Relaxed) <= skip_to {
        return;
    }
    let fps_desired = provider().tickrate() * BVAR_TAS_PLAYBACK_RATE.load(Ordering::Relaxed);
    if FPS_MAX.float_or(60.0) != fps_desired {
        FPS_MAX.set_float(fps_desired);
    }
}

fn exec_tas_start(start: &StartMethod) {
    match start {
        StartMethod::Now | StartMethod::NextNow => (),
        StartMethod::Save(save_name) | StartMethod::NextSave(save_name) => {
            if !is_save_exists(save_name) {
                clog!(err_prefix;
                    [Color::WHITE => "Specified save "],
                    [Color::RED => "{save_name}"],
                    [Color::WHITE => " doesn't exist"],
                );
                return;
            }
            cexec!("load {save_name}");
        }
        StartMethod::Map(map_name) | StartMethod::NextMap(map_name) => {
            if !is_map_exists(map_name) {
                clog!(err_prefix;
                    [Color::WHITE => "Specified map "],
                    [Color::RED => "{map_name}"],
                    [Color::WHITE => " doesn't exist"],
                );
                return;
            }
            // we use `map` instead of restart/changelevel
            // to improve consistency of TAS playback
            cexec!("map {map_name}");
        }
    }
}

fn play_tas(filename: String) {
    let extension = provider().tas_extension();
    let filename = if filename.ends_with(extension) {
        filename
    } else {
        filename + extension
    };

    let tas_path = format!("./tas/{filename}");
    *LAST_PLAYED_TAS.write().unwrap() = Some(tas_path.as_str().into());

    let parsed_tas = match parser::read_tas(tas_path) {
        Ok(tas) => tas,
        Err(TasError::IoError(error)) => {
            clog!(err_prefix;
                [Color::WHITE => "Failed to open tas script!\n"],
                [Color::RED => "Error:"],
                [Color::WHITE => " {error}"],
            );
            return;
        }
        Err(TasError::ParsingError {
            line_number,
            line,
            error,
        }) => {
            if line.is_empty() {
                clog!(err_prefix;
                    [Color::WHITE => "Failed to parse tas script!\n"],
                    [Color::RED => "File:"],
                    [Color::WHITE => " ./tas/{filename}\n"],
                    [Color::RED => "Error:"],
                    [Color::WHITE => " {error}"],
                );
            } else {
                clog!(err_prefix;
                    [Color::WHITE => "Failed to parse tas script!\n"],
                    [Color::RED => "File:"],
                    [Color::WHITE => " ./tas/{filename}\n"],
                    [Color::RED => " {line_number} |"],
                    [Color::WHITE => " {line}\n"],
                    [Color::RED => "Error:"],
                    [Color::WHITE => " {error}"],
                );
            }
            return;
        }
    };

    if parsed_tas.framebulks.is_empty() {
        clog!(err_prefix; "TAS file does not contain any framebulks");
        return;
    }
    let skip_to = BVAR_TAS_SKIP_TO.load(Ordering::Relaxed);
    let ticks_count = parsed_tas.framebulks.last().unwrap().tick + 1;
    if skip_to >= ticks_count {
        clog!(err_prefix;
            [Color::WHITE => "bvar_tas_skip_to is set to "],
            [Color::YELLOW => "{skip_to}"],
            [Color::WHITE => ", but provided TAS ends after "],
            [Color::YELLOW => "{ticks_count}"],
            [Color::WHITE => " {}", if ticks_count == 1 { "tick" } else { "ticks" }],
        );
        return;
    }

    exec_tas_start(&parsed_tas.start);

    match parsed_tas.start {
        StartMethod::Now | StartMethod::Map(_) | StartMethod::Save(_) => {
            SESSION_COUNTER.store(2, Ordering::Relaxed);
        }
        StartMethod::NextNow | StartMethod::NextMap(_) | StartMethod::NextSave(_) => {
            SESSION_COUNTER.store(3, Ordering::Relaxed);
        }
    }

    if !filename.ends_with(&("_raw".to_string() + extension)) {
        let raw_filename = if filename.ends_with(extension) {
            filename[..filename.len() - extension.len()].to_string() + "_raw" + extension
        } else {
            format!("{filename}_raw{extension}")
        };
        if let Ok(logger) = RawLogger::new(format!("./tas/{raw_filename}"), &parsed_tas) {
            RAW_LOGGER.write().unwrap().replace(logger);
        }
    }

    *PARSED_TAS.write().unwrap() = Some(parsed_tas);
}

/// Plays a TAS script with given name inside `/tas` folder.
///
/// `filename:` name of the file
#[burt_command(cheat)]
pub fn burt_tas_play(filename: String) {
    if TIME_PAUSED.load(Ordering::Relaxed) {
        clog!(prefix; "Time is paused, can't start the TAS");
        return;
    }
    if SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        end_tas();
    }
    play_tas(filename);
}

#[burt_autocomplete]
pub fn burt_tas_play_autocomplete(partial: String, writer: &mut AutocompleteWriter) {
    let query = partial
        .to_lowercase()
        .strip_prefix("burt_tas_play")
        .unwrap_or(&partial)
        .trim_matches(|c: char| c.is_whitespace() || c == '"')
        .replace('\\', "/");

    let (mut path_part, prefix) = query.rsplit_once('/').unwrap_or(("", query.as_str()));
    path_part = path_part.trim_end_matches('/');
    let search_dir = Path::new("./tas").join(path_part);

    let Ok(entries) = fs::read_dir(search_dir) else {
        return;
    };

    let extension = provider().tas_extension();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(file_name) = file_name.strip_suffix(extension) else {
            if !entry.path().is_dir() || !file_name.starts_with(prefix) {
                continue;
            }
            let mut path = if path_part.is_empty() {
                format!("{}/", file_name)
            } else {
                format!("{}/{}/", path_part, file_name)
            };

            if path.contains(' ') {
                path.insert(0, '"');
                path.push('"');
            }

            let completion = format!("burt_tas_play {path}");
            if let Err(AutocompleteError::TooManyCommands) = writer.push(&completion) {
                break;
            }
            continue;
        };

        if !file_name.starts_with(prefix) || entry.path().is_dir() {
            continue;
        }

        let mut path = format!(
            "{path_part}{}{file_name}",
            if !path_part.is_empty() { "/" } else { "" },
        );

        if path.contains(' ') {
            path.insert(0, '"');
            path.push('"');
        }

        let completion = format!("burt_tas_play {path}");

        if let Err(AutocompleteError::TooManyCommands) = writer.push(&completion) {
            break;
        }
    }
}

/// Replays the last played TAS.
#[burt_command(cheat)]
pub fn burt_tas_replay() {
    if TIME_PAUSED.load(Ordering::Relaxed) {
        clog!(prefix; "Time is paused, can't start the TAS");
        return;
    }
    let filename = {
        let last_tas_read = LAST_PLAYED_TAS.read().unwrap();
        let Some(filename) = last_tas_read.as_deref() else {
            clog!(err_prefix; "No TAS was played before");
            return;
        };
        filename.to_string()
    };
    if SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        end_tas();
    }
    play_tas(filename);
}

/// Stops the current TAS.
#[burt_command(cheat)]
pub fn burt_tas_stop() {
    if SESSION_COUNTER.load(Ordering::Relaxed) != 1 {
        clog!(err_prefix; "No TAS is currently playing");
        return;
    }
    let tick = TICK.load(Ordering::Relaxed);
    end_tas();
    clog!(prefix;
        [Color::WHITE => "TAS interrupted after "],
        [Color::YELLOW => "{} ticks", tick - 1],
    );
}

/// Pauses the TAS playback.
/// Since this command pauses the whole game, it can be used outside TAS too.
#[burt_command]
pub fn burt_tas_pause() {
    TIME_PAUSED.store(true, Ordering::Relaxed);
    if SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        if engine_build::current_build() == EngineBuild::Project2008 {
            HOST_FRAMERATE.set_float(provider().tickrate() + 0.0001);
        }
        clog!(prefix;
            [Color::WHITE => "TAS paused at tick "],
            [Color::YELLOW => "{}", TICK.load(Ordering::Relaxed)],
        );
    }
}

/// Resumes the TAS playback.
/// Since this command resumes the whole game, it can be used outside TAS too.
#[burt_command]
pub fn burt_tas_resume() {
    TIME_PAUSED.store(false, Ordering::Relaxed);
    if SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        if engine_build::current_build() == EngineBuild::Project2008 {
            HOST_FRAMERATE.set_float(provider().tickrate());
        }
        clog!(prefix;
            [Color::WHITE => "TAS resumed at tick "],
            [Color::YELLOW => "{}", TICK.load(Ordering::Relaxed) - 1],
        );
    }
}

/// Advances TAS playback by the specified amount of ticks.
/// By default it's set to `2`.
///
/// Since this command advances the pausing of the whole game,
/// it can be used outside TAS too.
#[burt_command(cheat)]
pub fn burt_tas_advance(ticks: Option<u32>) {
    if !TIME_PAUSED.load(Ordering::Relaxed) {
        clog!(err_prefix; "Time isn't paused, can't advance");
        return;
    }
    let ticks = ticks.unwrap_or(2);
    TIME_ADVANCE_TICKS.store(ticks, Ordering::Relaxed);
    if SESSION_COUNTER.load(Ordering::Relaxed) == 1 {
        let current_tick = TICK.load(Ordering::Relaxed) - 1;
        clog!(prefix;
            [Color::WHITE => "TAS advanced by "],
            [Color::YELLOW => "{ticks}"],
            [Color::WHITE => " {}: ", if ticks == 1 { "tick" } else { "ticks" }],
            [Color::YELLOW => "{current_tick}"],
            [Color::WHITE => " -> "],
            [Color::YELLOW => "{}", current_tick + ticks],
        );
    }
}
