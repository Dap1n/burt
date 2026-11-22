use std::{
    borrow::Cow,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::RangeInclusive,
    path::Path,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use smallvec::SmallVec;

use crate::{
    commands::{
        autojump::DuckState,
        p2008::scale::ScaleStep,
        strafe::{
            math,
            p2008::{Mode, Params},
            state::{Direction, Params as BaseParams, Speed, StrafeType},
        },
    },
    engine_build::{self, EngineBuild},
    game::{cinput::Action, cvars::MoveSpeed, math::normalize_vec2},
    structs::Easing,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    #[inline]
    pub const fn to_bits(self) -> u64 {
        ((self.x.to_bits() as u64) << 32) | (self.y.to_bits() as u64)
    }

    #[inline]
    pub const fn from_bits(n: u64) -> Self {
        Self {
            x: f32::from_bits((n >> 32) as u32),
            y: f32::from_bits(n as u32),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeldButtons(pub u8);

impl HeldButtons {
    pub const JUMP: u8 = 1 << 0;
    pub const DUCK: u8 = 1 << 1;
    pub const USE: u8 = 1 << 2;
    pub const SLOWTIME: u8 = 1 << 3;
    pub const LMB: u8 = 1 << 4;
    pub const RMB: u8 = 1 << 5;

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn remove_mask(&mut self, mask: HeldButtons) {
        self.0 &= !mask.0;
    }

    #[inline]
    pub const fn set(&mut self, n: u8) {
        self.0 |= n;
    }

    #[inline]
    pub const fn remove(&mut self, n: u8) {
        self.0 &= !n;
    }

    #[inline]
    pub const fn jump(&self) -> bool {
        self.0 & Self::JUMP != 0
    }

    #[inline]
    pub const fn duck(&self) -> bool {
        self.0 & Self::DUCK != 0
    }

    #[inline]
    pub const fn use_(&self) -> bool {
        self.0 & Self::USE != 0
    }

    #[inline]
    pub const fn slowtime(&self) -> bool {
        self.0 & Self::SLOWTIME != 0
    }

    #[inline]
    pub const fn lmb(&self) -> bool {
        self.0 & Self::LMB != 0
    }

    #[inline]
    pub const fn rmb(&self) -> bool {
        self.0 & Self::RMB != 0
    }

    pub fn from_char(ch: char, build: EngineBuild) -> Option<HeldButtons> {
        let bit = match ch {
            'J' | 'j' => Self::JUMP,
            'D' | 'd' => Self::DUCK,
            'U' | 'u' => Self::USE,
            'S' | 's' if build == EngineBuild::July2009 => Self::SLOWTIME,
            'L' | 'l' if build == EngineBuild::Project2008 => Self::LMB,
            'B' | 'b' if build != EngineBuild::Project2008 => Self::LMB,
            'R' | 'r' if build == EngineBuild::Project2008 => Self::RMB,
            'O' | 'o' if build != EngineBuild::Project2008 => Self::RMB,
            _ => return None,
        };
        Some(HeldButtons(bit))
    }

    pub fn to_string(self, build: EngineBuild) -> String {
        match build {
            EngineBuild::Project2008 => format!(
                "{}{}{}{}{}",
                if self.jump() { 'J' } else { 'j' },
                if self.duck() { 'D' } else { 'd' },
                if self.use_() { 'U' } else { 'u' },
                if self.lmb() { 'L' } else { 'l' },
                if self.rmb() { 'R' } else { 'r' },
            ),
            EngineBuild::July2009 => format!(
                "{}{}{}{}{}{}",
                if self.jump() { 'J' } else { 'j' },
                if self.duck() { 'D' } else { 'd' },
                if self.use_() { 'U' } else { 'u' },
                if self.slowtime() { 'S' } else { 's' },
                if self.lmb() { 'B' } else { 'b' },
                if self.rmb() { 'O' } else { 'o' },
            ),
            _ => unimplemented!(),
        }
    }

    pub const fn get_n(action: Action) -> u8 {
        match action {
            Action::Jump => Self::JUMP,
            Action::Duck => Self::DUCK,
            Action::Use => Self::USE,
            Action::Slowtime => Self::SLOWTIME,
            Action::Attack => Self::LMB,
            Action::Attack2 => Self::RMB,
        }
    }

    pub fn update_by_char(&mut self, ch: char, build: EngineBuild) -> bool {
        let Some(button) = Self::from_char(ch, build) else {
            return false;
        };
        if ch.is_uppercase() {
            self.set(button.0);
        } else {
            self.remove(button.0);
        }
        true
    }

    pub fn parse_actions(&self) -> impl Iterator<Item = Action> + use<> {
        const ACTIONS: [Action; 6] = [
            Action::Jump,
            Action::Duck,
            Action::Use,
            Action::Slowtime,
            Action::Attack,
            Action::Attack2,
        ];
        let inner = self.0;
        (0u8..6).filter_map(move |i| (inner & (1 << i) != 0).then_some(ACTIONS[i as usize]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolChange {
    Enable(Tool),
    Disable(ToolKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolKind {
    Autoaim,
    Autojump,
    Absmov,
    Strafe,
    Decel,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Check {
        params: CheckParams,
    },
    Scale {
        step: ScaleStep,
    },
    Setang {
        pitch: f32,
        yaw: f32,
        time: u32,
        easing: Easing,
    },
    Autoaim {
        x: f32,
        y: f32,
        z: f32,
        time: u32,
        easing: Easing,
    },
    AutoaimEnt {
        name: Option<Box<str>>,
        index: u32,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        time: u32,
        easing: Easing,
    },
    Autojump {
        duck_state: DuckState,
    },
    Absmov {
        angle: f32,
        strength: f32,
    },
    ModernStrafe {
        params: BaseParams,
    },
    LegacyStrafe {
        params: Params,
    },
    Decel {
        speed: f32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Framebulk {
    pub tick: u32,
    pub movement: Option<Point2D>,
    pub angles: Option<Point2D>,
    pub buttons: HeldButtons,
    pub commands: Option<Box<str>>,
    pub tools: Option<SmallVec<[ToolChange; 3]>>,
}

impl Framebulk {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum StartMethod {
    Now,
    Save(Box<str>),
    Map(Box<str>),
    NextNow,
    NextSave(Box<str>),
    NextMap(Box<str>),
}

impl Display for StartMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Now => write!(f, "now"),
            Self::Save(save) => write!(f, "save {save}"),
            Self::Map(map) => write!(f, "map {map}"),
            Self::NextNow => write!(f, "next now"),
            Self::NextSave(save) => write!(f, "next save {save}"),
            Self::NextMap(map) => write!(f, "next map {map}"),
        }
    }
}

fn remove_comments(line: &mut String, comment_depth: &mut u32) {
    if *comment_depth == 0 && !line.as_bytes().contains(&b'/') {
        return;
    }

    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_basic_comment = false;

    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                in_basic_comment = true;
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                *comment_depth += 1;
            }
            '*' if chars.peek() == Some(&'/') => {
                chars.next();
                *comment_depth -= 1;
            }
            _ => {
                if !in_basic_comment && *comment_depth == 0 {
                    out.push(c);
                }
            }
        }
    }

    *line = out;
}

#[derive(Debug)]
pub enum TasError {
    IoError(io::Error),
    ParsingError {
        line_number: u32,
        line: Box<str>,
        error: Cow<'static, str>,
    },
}

type TasResult<T> = Result<T, TasError>;

pub const TAS_VERSIONS: RangeInclusive<u8> = 1..=1;

fn parse_version(line_number: u32, line: &str) -> TasResult<u8> {
    if line.starts_with("version") {
        let Some(ver) = line
            .strip_prefix("version")
            .map(str::trim)
            .and_then(|s| s.parse().ok())
        else {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Failed to parse version"),
            });
        };
        if TAS_VERSIONS.contains(&ver) {
            Ok(ver)
        } else {
            Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Owned(format!(
                    "Unsupported version {ver}. Available versions: {TAS_VERSIONS:?}"
                )),
            })
        }
    } else {
        Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Expected version"),
        })
    }
}

fn parse_start(line_number: u32, line: &str) -> TasResult<StartMethod> {
    let mut tokens_iter = line.split_whitespace();
    if tokens_iter.next() != Some("start") {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Expected start command"),
        });
    }
    match tokens_iter.next().unwrap() {
        "now" => {
            if tokens_iter.next().is_some() {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Unexpected token after now parameter"),
                });
            }
            Ok(StartMethod::Now)
        }
        "save" => {
            let Some(save_name) = tokens_iter.next() else {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("No save name provided"),
                });
            };
            if tokens_iter.next().is_some() {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Unexpected token after save name"),
                });
            }
            Ok(StartMethod::Save(save_name.into()))
        }
        "map" => {
            let Some(map_name) = tokens_iter.next() else {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("No map name provided"),
                });
            };
            if tokens_iter.next().is_some() {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Unexpected token after map name"),
                });
            }
            Ok(StartMethod::Map(map_name.into()))
        }
        "next" => match tokens_iter.next() {
            Some("now") => {
                if tokens_iter.next().is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Unexpected token after now parameter"),
                    });
                }
                Ok(StartMethod::NextNow)
            }
            None => Ok(StartMethod::NextNow),
            Some("save") => {
                let Some(save_name) = tokens_iter.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("No save name provided"),
                    });
                };
                if tokens_iter.next().is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Unexpected token after save name"),
                    });
                }
                Ok(StartMethod::NextSave(save_name.into()))
            }
            Some("map") => {
                let Some(map_name) = tokens_iter.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("No map name provided"),
                    });
                };
                if tokens_iter.next().is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Unexpected token after map name"),
                    });
                }
                Ok(StartMethod::NextMap(map_name.into()))
            }
            _ => Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed(
                    "Unexpected parameter after next parameter. Available: now, save, map",
                ),
            }),
        },
        _ => Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed(
                "Unexpected parameter after start command. Available: now, save, map, next",
            ),
        }),
    }
}

fn parse_point2d(
    point_raw: &str,
    is_movement: bool,
    line_number: u32,
    line: &str,
) -> TasResult<Point2D> {
    let mut tokens_iter = point_raw.split_whitespace();

    let x: f32 = tokens_iter
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| TasError::ParsingError {
            line: line.into(),
            line_number,
            error: Cow::Borrowed(if is_movement {
                "Can't parse x parameter in movement"
            } else {
                "Can't parse x parameter in angles"
            }),
        })?;
    let y: f32 = tokens_iter
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| TasError::ParsingError {
            line: line.into(),
            line_number,
            error: Cow::Borrowed(if is_movement {
                "Can't parse y parameter in movement"
            } else {
                "Can't parse y parameter in angles"
            }),
        })?;

    if tokens_iter.next().is_some() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed(if is_movement {
                "Expected exactly 2 tokens in movement"
            } else {
                "Expected exactly 2 tokens in angles"
            }),
        });
    }

    if is_movement {
        if !(-1.0..=1.0).contains(&x) {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("x parameter in movement must be in range from -1 to 1"),
            });
        }
        if !(-1.0..=1.0).contains(&y) {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("y parameter in movement must be in range from -1 to 1"),
            });
        }
    } else if !x.is_finite() || !y.is_finite() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Expected finite numbers in angles"),
        });
    }

    Ok(Point2D { x, y })
}

fn parse_buttons(
    tick: u32,
    buttons_raw: &str,
    previous_buttons: HeldButtons,
    scheduled: &mut Vec<(u32, HeldButtons)>,
    line_number: u32,
    line: &str,
) -> TasResult<HeldButtons> {
    let build = engine_build::current_build();
    let mut buttons = previous_buttons;
    let mut chars_iter = buttons_raw.chars().peekable();

    while let Some(ch) = chars_iter.next() {
        if !buttons.update_by_char(ch, build) {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Owned(format!("Unknown button: {ch}")),
            });
        }

        if chars_iter.peek().is_some_and(|c| c.is_ascii_digit()) {
            if ch.is_lowercase() {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Duration of button release isn't supported"),
                });
            }
            let mut duration = String::with_capacity(2);
            while let Some(digit) = chars_iter.next_if(|c| c.is_ascii_digit()) {
                duration.push(digit);
            }
            let duration = duration
                .parse::<u32>()
                .map_err(|_| TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Owned(format!("Can't parse duration of button {ch}")),
                })?;
            let future_tick = tick + duration;
            let release_mask = HeldButtons::from_char(ch, build).unwrap();
            scheduled.push((future_tick, release_mask));
        }
    }

    Ok(buttons)
}

fn upsert_tool(
    changes: &mut SmallVec<[ToolChange; 3]>,
    current_tools: &mut Vec<Tool>,
    is_match: &impl Fn(&Tool) -> bool,
    new_tool: Tool,
) {
    if let Some(idx) = current_tools.iter().position(is_match) {
        changes.push(ToolChange::Enable(new_tool.clone()));
        current_tools[idx] = new_tool;
    } else {
        changes.push(ToolChange::Enable(new_tool.clone()));
        current_tools.push(new_tool);
    }
}

fn upsert_or_disable_tool(
    changes: &mut SmallVec<[ToolChange; 3]>,
    current_tools: &mut Vec<Tool>,
    is_match: &impl Fn(&Tool) -> bool,
    new_tool: Option<Tool>,
    disable_kind: ToolKind,
) {
    if let Some(tool) = new_tool {
        upsert_tool(changes, current_tools, is_match, tool);
    } else if let Some(idx) = current_tools.iter().position(is_match) {
        changes.push(ToolChange::Disable(disable_kind));
        current_tools.remove(idx);
    }
}

fn parse_tools(
    tools_raw: &str,
    current_tools: &mut Vec<Tool>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<SmallVec<[ToolChange; 3]>>> {
    let mut changes: SmallVec<[ToolChange; 3]> = SmallVec::new();

    for tool_raw in tools_raw
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let mut tokens = tool_raw.split_whitespace();
        let Some(name) = tokens.next() else { continue };

        match name {
            "check" => {
                let params = parse_check_tokens(&mut tokens, line_number, line)?;
                upsert_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Check { .. }),
                    Tool::Check { params },
                );
            }
            "scale" => {
                let step = parse_scale_tokens(&mut tokens, line_number, line)?;
                upsert_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Scale { .. }),
                    Tool::Scale { step },
                );
            }
            "setang" => {
                let (pitch, yaw, time, easing) =
                    parse_setang_tokens(&mut tokens, line_number, line)?;
                upsert_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Setang { .. }),
                    Tool::Setang {
                        pitch,
                        yaw,
                        time,
                        easing,
                    },
                );
            }
            "autoaim" => {
                let parsed = parse_autoaim_tokens(&mut tokens, line_number, line)?;
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Autoaim { .. } | Tool::AutoaimEnt { .. }),
                    parsed,
                    ToolKind::Autoaim,
                );
            }
            "autojump" => {
                let duck_state = parse_autojump_tokens(&mut tokens, line_number, line)?;
                if duck_state == Some(DuckState::Full)
                    && engine_build::current_build() != EngineBuild::Project2008
                {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Attempted to use full ducking outside Project 2008"),
                    });
                }
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Autojump { .. }),
                    duck_state.map(|duck_state| Tool::Autojump { duck_state }),
                    ToolKind::Autojump,
                );
            }
            "absmov" => {
                let parsed = parse_absmov_tokens(&mut tokens, line_number, line)?;
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Absmov { .. }),
                    parsed.map(|(angle, strength)| Tool::Absmov { angle, strength }),
                    ToolKind::Absmov,
                );
            }
            "strafe" if engine_build::current_build() == EngineBuild::Project2008 => {
                let params = parse_legacy_strafe_tokens(&mut tokens, line_number, line)?;
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::LegacyStrafe { .. }),
                    params.map(|params| Tool::LegacyStrafe { params }),
                    ToolKind::Strafe,
                );
            }
            "strafe" => {
                let params = parse_modern_strafe_tokens(&mut tokens, line_number, line)?;
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::ModernStrafe { .. }),
                    params.map(|params| Tool::ModernStrafe { params }),
                    ToolKind::Strafe,
                );
            }
            "decel" => {
                let parsed = parse_decel_tokens(&mut tokens, line_number, line)?;
                upsert_or_disable_tool(
                    &mut changes,
                    current_tools,
                    &|t| matches!(t, Tool::Decel { .. }),
                    parsed.map(|speed| Tool::Decel { speed }),
                    ToolKind::Decel,
                );
            }
            other => {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Owned(format!("Unknown tool: {other}")),
                });
            }
        }
    }

    if changes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(changes))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityCheck {
    NotChecked,
    Anything,
    Named(Box<str>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckParams {
    pub pos: Option<[f32; 3]>,
    pub ang: Option<[f32; 2]>,
    pub vel: Option<f32>,
    pub holding: EntityCheck,
    pub captured: EntityCheck,
    pub posepsilon: f32,
    pub angepsilon: f32,
    pub velepsilon: f32,
}

fn parse_check_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<CheckParams> {
    let mut tokens = tokens.peekable();

    let is_next_keyword = |next: &_| {
        [
            "pos",
            "ang",
            "vel",
            "holding",
            "captured",
            "posepsilon",
            "angepsilon",
            "velepsilon",
        ]
        .contains(next)
    };

    let mut pos = None;
    let mut ang = None;
    let mut vel = None;
    let mut holding = EntityCheck::NotChecked;
    let mut captured = EntityCheck::NotChecked;
    let mut posepsilon = 0.5;
    let mut angepsilon = 0.2;
    let mut velepsilon = 1.0;

    while let Some(token) = tokens.next() {
        match token {
            "pos" => {
                if pos.is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Duplicate position given to check tool"),
                    });
                }
                let mut parse_float = || {
                    tokens
                        .next()
                        .and_then(|s| s.parse::<f32>().ok())
                        .filter(|f| f.is_finite())
                };
                let Some(x) = parse_float() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Bad position for check tool"),
                    });
                };
                let Some(y) = parse_float() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Bad position for check tool"),
                    });
                };
                let Some(z) = parse_float() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Bad position for check tool"),
                    });
                };
                pos = Some([x, y, z]);
            }
            "ang" => {
                if ang.is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Duplicate angle given to check tool"),
                    });
                }
                let mut parse_float = || {
                    tokens
                        .next()
                        .and_then(|s| s.parse::<f32>().ok())
                        .filter(|f| f.is_finite())
                };
                let Some(p) = parse_float() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Bad angle for check tool"),
                    });
                };
                let Some(y) = parse_float() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Bad angle for check tool"),
                    });
                };
                ang = Some([p, y]);
            }
            "vel" => {
                if vel.is_some() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Duplicate velocity given to check tool"),
                    });
                }
                let Some(token) = tokens.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Missing velocity value for check tool"),
                    });
                };
                let val_str = token.strip_suffix("ups").unwrap_or(token);
                let speed = val_str.parse::<f32>().map_err(|_| TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Bad velocity value"),
                })?;
                if !speed.is_finite() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Velocity is not finite"),
                    });
                }
                vel = Some(speed);
            }
            "holding" => {
                if holding != EntityCheck::NotChecked {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Duplicate holding given to check tool"),
                    });
                }

                let is_keyword = tokens.peek().is_none_or(is_next_keyword);
                if is_keyword {
                    holding = EntityCheck::Anything;
                } else {
                    holding = EntityCheck::Named(tokens.next().unwrap().into());
                }
            }
            "captured" => {
                if engine_build::current_build() != EngineBuild::Project2008 {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Attempted to use capture check outside Project 2008"),
                    });
                }
                if captured != EntityCheck::NotChecked {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Duplicate captured given to check tool"),
                    });
                }

                let is_keyword = tokens.peek().is_none_or(is_next_keyword);
                if is_keyword {
                    captured = EntityCheck::Anything;
                } else {
                    captured = EntityCheck::Named(tokens.next().unwrap().into());
                }
            }
            "posepsilon" => {
                let Some(val_str) = tokens.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Missing posepsilon value"),
                    });
                };
                posepsilon = val_str.parse::<f32>().map_err(|_| TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Bad posepsilon value"),
                })?;
                if !posepsilon.is_finite() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("posepsilon is not finite"),
                    });
                }
            }
            "angepsilon" => {
                let Some(val_str) = tokens.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Missing angepsilon value"),
                    });
                };
                angepsilon = val_str.parse::<f32>().map_err(|_| TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Bad angepsilon value"),
                })?;
                if !angepsilon.is_finite() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("angepsilon is not finite"),
                    });
                }
            }
            "velepsilon" => {
                let Some(val_str) = tokens.next() else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Missing velepsilon value"),
                    });
                };
                velepsilon = val_str.parse::<f32>().map_err(|_| TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed("Bad velepsilon value"),
                })?;
                if !velepsilon.is_finite() {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("velepsilon is not finite"),
                    });
                }
            }
            _ => {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Owned(format!("Unknown check param: {token}")),
                });
            }
        }
    }

    Ok(CheckParams {
        pos,
        ang,
        vel,
        holding,
        captured,
        posepsilon,
        angepsilon,
        velepsilon,
    })
}

fn parse_scale_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<ScaleStep> {
    if engine_build::current_build() != EngineBuild::Project2008 {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Requested scale tool outside Project 2008"),
        });
    }

    let token = tokens.next().ok_or_else(|| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Borrowed("Expected a scale step"),
    })?;

    if let Ok(step) = token.parse::<i32>() {
        return Ok(ScaleStep::Exact(step));
    }

    match token {
        "min" => Ok(ScaleStep::Min),
        "max" => Ok(ScaleStep::Max),

        "up" | "down" => {
            let sign = if token == "up" { 1 } else { -1 };

            let Some(next) = tokens.next() else {
                return Ok(ScaleStep::Relative(sign));
            };
            let Ok(n @ 1..) = next.parse() else {
                return Err(TasError::ParsingError {
                    line_number,
                    line: line.into(),
                    error: Cow::Borrowed(
                        "Expected a parseable signed 32-bit non-zero integer after up/down",
                    ),
                });
            };
            Ok(ScaleStep::Relative(n * sign))
        }

        _ => Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Expected a parseable signed 32-bit integer, min/max or up/down"),
        }),
    }
}

fn parse_legacy_strafe_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<Params>> {
    let mut params = Params::new();

    while let Some(token) = tokens.next() {
        match token {
            "off" | "none" => return Ok(None),
            "abh" => params.mode = Mode::Abh,
            "afh" => params.mode = Mode::Afh,
            "forward" => {
                params.dir = Direction::Current {
                    use_vel_angle: false,
                };
            }
            "forwardvel" => {
                params.dir = Direction::Current {
                    use_vel_angle: true,
                };
            }
            "left" => {
                params.dir = Direction::Specified(10000.0);
            }
            "right" => {
                params.dir = Direction::Specified(-10000.0);
            }
            "afhlength" => {
                let Some(afh_length) = tokens.next().and_then(|s| s.parse().ok()) else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Expected parseable float after afhlength"),
                    });
                };
                if !(0.0..=1.0).contains(&afh_length) {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed(
                            "Expected afhlength to be a number in range from 0 to 1",
                        ),
                    });
                }
                params.afh_length = afh_length;
            }
            _ => {
                if let Some(s) = token.strip_suffix("deg") {
                    let v: f32 = s.parse().map_err(|_| TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Owned(format!("Invalid deg: {token}")),
                    })?;
                    if !v.is_finite() {
                        return Err(TasError::ParsingError {
                            line_number,
                            line: line.into(),
                            error: Cow::Borrowed("Deg is not finite"),
                        });
                    }
                    params.dir = Direction::Specified(math::normalize_signed(v));
                } else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Owned(format!("Unknown strafe param: {token}")),
                    });
                }
            }
        }
    }

    Ok(Some(params))
}

fn parse_modern_strafe_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<BaseParams>> {
    let mut params = BaseParams::new();

    for token in tokens {
        match token {
            "off" | "none" => return Ok(None),
            "vec" => {
                params.type_ = StrafeType::Vectorial;
            }
            "ang" | "angular" => {
                params.type_ = StrafeType::Angular;
            }
            "veccam" | "vectorialcam" => {
                params.type_ = StrafeType::VectorialCam;
            }
            "forward" => {
                params.dir = Direction::Current {
                    use_vel_angle: false,
                };
            }
            "forwardvel" => {
                params.dir = Direction::Current {
                    use_vel_angle: true,
                };
            }
            "left" => {
                params.dir = Direction::Specified(10000.0);
            }
            "right" => {
                params.dir = Direction::Specified(-10000.0);
            }
            "max" => {
                params.speed = Speed::Max;
            }
            "keep" => {
                params.speed = Speed::Current;
            }
            "nopitchlock" => {
                params.no_pitch_lock = true;
            }
            "letspeedlock" => {
                params.let_speed_lock = true;
            }
            _ => {
                if let Some(s) = token.strip_suffix("deg") {
                    let v: f32 = s.parse().map_err(|_| TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Owned(format!("Invalid deg: {token}")),
                    })?;
                    if !v.is_finite() {
                        return Err(TasError::ParsingError {
                            line_number,
                            line: line.into(),
                            error: Cow::Borrowed("Deg is not finite"),
                        });
                    }
                    params.dir = Direction::Specified(math::normalize_signed(v));
                } else if let Some(s) = token.strip_suffix("ups") {
                    let v: f32 = s.parse().map_err(|_| TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Borrowed("Invalid ups"),
                    })?;
                    if v <= 0.0 || !v.is_finite() {
                        return Err(TasError::ParsingError {
                            line_number,
                            line: line.into(),
                            error: Cow::Borrowed("Invalid target speed"),
                        });
                    }
                    params.speed = Speed::Specified(v);
                } else {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: line.into(),
                        error: Cow::Owned(format!("Unknown strafe param: {token}")),
                    });
                }
            }
        }
    }

    Ok(Some(params))
}

fn parse_autojump_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<DuckState>> {
    for (i, token) in tokens.enumerate() {
        if i != 0 {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Expected not more than 1 token in autojump"),
            });
        }
        if token == "off" {
            return Ok(None);
        }
        return Ok(Some(DuckState::from_str(token)));
    }
    Ok(Some(DuckState::Disabled))
}

fn parse_absmov_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<(f32, f32)>> {
    let Some(first) = tokens.next() else {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Absmov requires an angle"),
        });
    };
    if first == "off" {
        return Ok(None);
    }
    let angle_str = first.strip_suffix("deg").unwrap_or(first);
    let angle: f32 = angle_str.parse().map_err(|_| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Owned(format!("Invalid absmov angle: {first}")),
    })?;
    if !angle.is_finite() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Angle is not finite"),
        });
    }
    let strength = tokens.next().map_or(Ok(1.0), |s| {
        s.parse::<f32>().map_err(|_| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Owned(format!("Invalid absmov strength: {s}")),
        })
    })?;
    if !(0.0..=1.0).contains(&strength) {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Strength is not in range from 0 to 1"),
        });
    }
    Ok(Some((math::normalize_signed(angle), strength)))
}

fn parse_setang_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<(f32, f32, u32, Easing)> {
    let pitch_str = tokens.next().ok_or_else(|| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Borrowed("Setang requires pitch"),
    })?;
    let pitch: f32 = pitch_str.parse().map_err(|_| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Owned(format!("Invalid setang pitch: {pitch_str}")),
    })?;
    if !(-89.0..=89.0).contains(&pitch) {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Pitch must be in range from -89 to 89"),
        });
    }
    let yaw_str = tokens.next().ok_or_else(|| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Borrowed("Setang requires yaw"),
    })?;
    let yaw: f32 = yaw_str.parse().map_err(|_| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Owned(format!("Invalid setang yaw: {yaw_str}")),
    })?;
    if !yaw.is_finite() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Yaw is not finite"),
        });
    }
    let time: u32 = tokens.next().map_or(Ok(0), |s| {
        s.parse().map_err(|_| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Owned(format!("Invalid setang time: {s}")),
        })
    })?;
    let easing = tokens.next().map_or(Ok(Easing::Linear), |s| {
        Easing::parse(s).ok_or_else(|| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Owned(format!("Invalid setang easing: {s}")),
        })
    })?;
    Ok((pitch, math::normalize_signed(yaw), time, easing))
}

fn parse_autoaim_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<Tool>> {
    let Some(first) = tokens.next() else {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Autoaim requires x"),
        });
    };

    if first == "off" {
        return Ok(None);
    }

    if first == "ent" {
        let target_str = tokens.next().ok_or_else(|| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Autoaim ent requires a target"),
        })?;

        let (name, index) = if let Some(bracket_start) = target_str.find('[') {
            let bracket_end = target_str.find(']').ok_or_else(|| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid entity selector"),
            })?;
            let name = &target_str[..bracket_start];
            let n_str = &target_str[bracket_start + 1..bracket_end];
            let idx: u32 = n_str.parse().map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid entity index in selector"),
            })?;
            (Some(name.into()), idx)
        } else if let Ok(idx) = target_str.parse::<u32>() {
            (None, idx)
        } else {
            (Some(target_str.into()), 0)
        };

        let offset_x: f32 = tokens.next().map_or(Ok(0.0), |s| {
            s.parse().map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid x offset"),
            })
        })?;
        let offset_y: f32 = tokens.next().map_or(Ok(0.0), |s| {
            s.parse().map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid y offset"),
            })
        })?;
        let offset_z: f32 = tokens.next().map_or(Ok(0.0), |s| {
            s.parse().map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid z offset"),
            })
        })?;
        if !offset_x.is_finite() || !offset_y.is_finite() || !offset_z.is_finite() {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Non-finite numbers detected in origin offsets"),
            });
        }

        let time = tokens.next().map_or(Ok(0), |s| {
            s.parse().map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Invalid autoaim time"),
            })
        })?;

        let easing = tokens.next().map_or(Ok(Easing::Linear), |s| {
            Easing::parse(s).ok_or_else(|| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Owned(format!("Invalid autoaim easing: {s}")),
            })
        })?;

        return Ok(Some(Tool::AutoaimEnt {
            name,
            index,
            offset_x,
            offset_y,
            offset_z,
            time,
            easing,
        }));
    }

    let x: f32 = first.parse().map_err(|_| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Borrowed("Invalid autoaim x"),
    })?;
    let y: f32 = tokens
        .next()
        .ok_or_else(|| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Autoaim requires y"),
        })?
        .parse()
        .map_err(|_| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Invalid autoaim y"),
        })?;
    let z: f32 = tokens
        .next()
        .ok_or_else(|| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Autoaim requires z"),
        })?
        .parse()
        .map_err(|_| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Invalid autoaim z"),
        })?;
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Non-finite numbers detected in target coordinates"),
        });
    }

    let time = tokens.next().map_or(Ok(0), |s| {
        s.parse().map_err(|_| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Invalid autoaim time"),
        })
    })?;
    let easing = tokens.next().map_or(Ok(Easing::Linear), |s| {
        Easing::parse(s).ok_or_else(|| TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Invalid autoaim easing"),
        })
    })?;

    Ok(Some(Tool::Autoaim {
        x,
        y,
        z,
        time,
        easing,
    }))
}

fn parse_decel_tokens<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    line_number: u32,
    line: &str,
) -> TasResult<Option<f32>> {
    let Some(first) = tokens.next() else {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Decel requires a speed"),
        });
    };
    if first == "off" {
        return Ok(None);
    }
    let speed_str = first.strip_suffix("ups").unwrap_or(first);
    let speed: f32 = speed_str.parse().map_err(|_| TasError::ParsingError {
        line_number,
        line: line.into(),
        error: Cow::Owned(format!("Invalid decel speed: {first}")),
    })?;
    if speed < 0.0 || !speed.is_finite() {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Invalid decel speed"),
        });
    }
    Ok(Some(speed))
}

fn release_scheduled(
    tick: Option<u32>,
    scheduled: &mut Vec<(u32, HeldButtons)>,
    buttons: &mut HeldButtons,
    framebulks: &mut Vec<Framebulk>,
) {
    let mut removed: Vec<_> = scheduled
        .extract_if(.., |(sch_tick, _)| {
            tick.is_none_or(|curr_tick| *sch_tick < curr_tick)
        })
        .collect();

    removed.sort_by_key(|(sch_tick, _)| *sch_tick);

    for (sch_tick, release_mask) in removed {
        buttons.remove_mask(release_mask);

        if let Some(existing) = framebulks.iter_mut().find(|fb| fb.tick == sch_tick) {
            existing.buttons.remove_mask(release_mask);
        } else {
            let position = framebulks
                .iter()
                .position(|fb| fb.tick > sch_tick)
                .unwrap_or(framebulks.len());

            framebulks.insert(
                position,
                Framebulk {
                    tick: sch_tick,
                    buttons: *buttons,
                    ..Default::default()
                },
            );
        }
    }
}

struct ParseState<'a> {
    is_first: &'a mut bool,
    current_tick: &'a mut u32,
    current_tools: &'a mut Vec<Tool>,
    scheduled: &'a mut Vec<(u32, HeldButtons)>,
    buttons: &'a mut HeldButtons,
    framebulks: &'a mut Vec<Framebulk>,
}

fn parse_framebulk(
    state: &mut ParseState<'_>,
    line_number: u32,
    line: &str,
) -> TasResult<Framebulk> {
    let Some((new_tick, fb_raw)) = line.split_once('>') else {
        return Err(TasError::ParsingError {
            line_number,
            line: line.into(),
            error: Cow::Borrowed("Can't find > symbol in framebulk"),
        });
    };

    if let Some(stripped) = new_tick.strip_prefix('+') {
        *state.current_tick += stripped
            .parse::<u32>()
            .map_err(|_| TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Can't parse the tick"),
            })?;
    } else {
        let Ok(new_tick) = new_tick.parse() else {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Borrowed("Can't parse the tick"),
            });
        };
        if new_tick <= *state.current_tick && !*state.is_first {
            return Err(TasError::ParsingError {
                line_number,
                line: line.into(),
                error: Cow::Owned(format!(
                    "Next tick must be greater than previous ({new_tick} <= {})",
                    *state.current_tick
                )),
            });
        }
        *state.current_tick = new_tick;
    }

    let mut framebulk = Framebulk::new();
    framebulk.tick = *state.current_tick;
    *state.is_first = false;

    release_scheduled(
        Some(*state.current_tick),
        state.scheduled,
        state.buttons,
        state.framebulks,
    );
    framebulk.buttons = *state.buttons;

    if let Some(stripped) = fb_raw.strip_prefix('>') {
        framebulk.tools = parse_tools(stripped, state.current_tools, line_number, line)?;
        return Ok(framebulk);
    }

    let mut tokens = fb_raw.split('|');
    if let Some(movement) = tokens.next().map(str::trim)
        && !movement.is_empty()
    {
        framebulk.movement = Some(parse_point2d(movement, true, line_number, line)?);
    }
    if let Some(angles) = tokens.next().map(str::trim)
        && !angles.is_empty()
    {
        framebulk.angles = Some(parse_point2d(angles, false, line_number, line)?);
    }
    if let Some(buttons_raw) = tokens.next().map(str::trim)
        && !buttons_raw.is_empty()
    {
        framebulk.buttons = parse_buttons(
            *state.current_tick,
            buttons_raw,
            *state.buttons,
            state.scheduled,
            line_number,
            line,
        )?;
    }
    if let Some(commands_raw) = tokens.next().map(str::trim)
        && !commands_raw.is_empty()
    {
        framebulk.commands = Some(commands_raw.into());
    }
    if let Some(tools_raw) = tokens.next().map(str::trim)
        && !tools_raw.is_empty()
    {
        framebulk.tools = parse_tools(tools_raw, state.current_tools, line_number, line)?;
    }

    *state.buttons = framebulk.buttons;
    Ok(framebulk)
}

#[derive(Debug, Clone)]
pub struct TasScript {
    pub version: u8,
    pub start: StartMethod,
    pub framebulks: Vec<Framebulk>,
}

#[derive(Default)]
pub struct FramebulkIntent<'a> {
    pub angles: Option<Point2D>,
    pub actions_press: Option<Box<dyn Iterator<Item = Action> + 'a>>,
    pub actions_release: Option<Box<dyn Iterator<Item = Action> + 'a>>,
    pub commands: Option<&'a str>,
    pub tools: Option<&'a [ToolChange]>,
}

impl TasScript {
    pub fn get_intent<'a>(
        &'a self,
        tick: u32,
        skip: &AtomicUsize,
        current_movement: &AtomicU64,
        current_angles: &AtomicU64,
    ) -> Option<FramebulkIntent<'a>> {
        let skip_inner = skip.load(Ordering::Relaxed);
        let fb1 = self.framebulks.get(skip_inner)?;
        if tick < fb1.tick {
            return Some(FramebulkIntent::default());
        }
        if tick == fb1.tick {
            if let Some(movement) = fb1.movement
                && let Some((x, y)) =
                    normalize_vec2((movement.x, movement.y), MoveSpeed::with_ones())
            {
                let point = Point2D { x, y };
                current_movement.store(point.to_bits(), Ordering::Relaxed);
            }
            if let Some(angles) = fb1.angles {
                current_angles.store(angles.to_bits(), Ordering::Relaxed);
            }
            return Some(FramebulkIntent {
                angles: fb1.angles,
                actions_press: (!fb1.buttons.is_empty())
                    .then(|| Box::new(fb1.buttons.parse_actions()) as Box<_>),
                actions_release: None,
                commands: fb1.commands.as_deref(),
                tools: fb1.tools.as_ref().map(|x| x.as_slice()),
            });
        }
        let fb2 = self.framebulks.get(skip_inner + 1)?;
        if tick < fb2.tick {
            let angles = Point2D::from_bits(current_angles.load(Ordering::Relaxed));
            return Some(FramebulkIntent {
                angles: (!angles.is_zero()).then_some(angles),
                ..Default::default()
            });
        }
        skip.fetch_add(1, Ordering::Relaxed);
        if let Some(movement) = fb2.movement {
            current_movement.store(movement.to_bits(), Ordering::Relaxed);
        }
        if let Some(angles) = fb2.angles {
            current_angles.store(angles.to_bits(), Ordering::Relaxed);
        }
        let press = HeldButtons(fb2.buttons.0 & !fb1.buttons.0);
        let release = HeldButtons(fb1.buttons.0 & !fb2.buttons.0);
        let angles = Point2D::from_bits(current_angles.load(Ordering::Relaxed));
        Some(FramebulkIntent {
            angles: (!angles.is_zero()).then_some(angles),
            actions_press: (!press.is_empty()).then(|| Box::new(press.parse_actions()) as Box<_>),
            actions_release: (!release.is_empty())
                .then(|| Box::new(release.parse_actions()) as Box<_>),
            commands: fb2.commands.as_deref(),
            tools: fb2.tools.as_ref().map(|x| x.as_slice()),
        })
    }
}

pub fn read_tas(path: impl AsRef<Path>) -> TasResult<TasScript> {
    let file = File::open(path.as_ref()).map_err(TasError::IoError)?;
    let mut reader = BufReader::new(file);
    let mut line_buf = String::new();
    let mut line_number = 0;
    let mut version = None;
    let mut start = None;
    let mut comment_depth = 0;

    let mut is_first = true;
    let mut current_tick = 0;
    let mut current_tools = Vec::new();
    let mut scheduled = Vec::new();
    let mut buttons = HeldButtons::default();
    let mut framebulks = Vec::new();

    let mut repeat_stack = Vec::new();

    {
        let mut state = ParseState {
            is_first: &mut is_first,
            current_tick: &mut current_tick,
            current_tools: &mut current_tools,
            scheduled: &mut scheduled,
            buttons: &mut buttons,
            framebulks: &mut framebulks,
        };

        loop {
            line_buf.clear();
            let n = reader.read_line(&mut line_buf).map_err(TasError::IoError)?;
            if n == 0 {
                break;
            }
            line_number += 1;
            remove_comments(&mut line_buf, &mut comment_depth);
            let trimmed = line_buf.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            if version.is_none() {
                version = Some(parse_version(line_number, &trimmed)?);
                continue;
            }
            if start.is_none() {
                start = Some(parse_start(line_number, &trimmed)?);
                continue;
            }

            if let Some(count_str) = trimmed.strip_prefix("repeat ") {
                let count = count_str
                    .trim()
                    .parse()
                    .map_err(|_| TasError::ParsingError {
                        line_number,
                        line: trimmed.clone().into_boxed_str(),
                        error: Cow::Borrowed("Invalid repeat count"),
                    })?;
                if count <= 1 {
                    return Err(TasError::ParsingError {
                        line_number,
                        line: trimmed.clone().into_boxed_str(),
                        error: Cow::Borrowed("Expected repeat count larger than 1"),
                    });
                }
                repeat_stack.push((line_number, count, Vec::new()));
                continue;
            }
            if trimmed == "end" {
                if let Some((start_line, count, buffered)) = repeat_stack.pop() {
                    if let Some(parent) = repeat_stack.last_mut() {
                        parent.2.push((start_line, format!("repeat {count}")));
                        parent.2.extend(buffered);
                        parent.2.push((line_number, "end".to_string()));
                    } else {
                        apply_repeat_block(&buffered, count, &mut state)?;
                    }
                }
                continue;
            }

            if let Some(top) = repeat_stack.last_mut() {
                top.2.push((line_number, trimmed.clone()));
                continue;
            }

            let fb = parse_framebulk(&mut state, line_number, &trimmed)?;
            state.framebulks.push(fb);
        }

        if !repeat_stack.is_empty() {
            return Err(TasError::ParsingError {
                line_number,
                line: "".into(),
                error: Cow::Borrowed("Unclosed repeat block"),
            });
        }

        release_scheduled(None, state.scheduled, state.buttons, state.framebulks);
    }

    buttons = HeldButtons::default();
    let last_framebulk = framebulks.last().cloned();
    framebulks.retain(|fb| {
        let result = fb.movement.is_some()
            || fb.angles.is_some()
            || fb.buttons != buttons
            || fb.commands.is_some()
            || fb.tools.is_some();
        buttons = fb.buttons;
        result
    });
    if let Some(fb) = last_framebulk
        && let Some(new_last_fb) = framebulks.last()
        && fb.angles.is_none()
        && fb.movement.is_none()
        && fb.buttons == new_last_fb.buttons
        && fb.commands.is_none()
        && fb.tools.is_none()
    {
        framebulks.push(fb);
    }

    Ok(TasScript {
        version: version.ok_or(TasError::ParsingError {
            line_number,
            line: "".into(),
            error: Cow::Borrowed("No version is provided"),
        })?,
        start: start.ok_or(TasError::ParsingError {
            line_number,
            line: "".into(),
            error: Cow::Borrowed("No start method is provided"),
        })?,
        framebulks,
    })
}

fn apply_repeat_block(
    block: &[(u32, String)],
    count: u32,
    state: &mut ParseState<'_>,
) -> TasResult<()> {
    for _ in 0..count {
        let mut i = 0;
        while i < block.len() {
            let (ln, text) = &block[i];

            if let Some(count_str) = text.strip_prefix("repeat ") {
                let n: u32 = count_str
                    .trim()
                    .parse()
                    .map_err(|_| TasError::ParsingError {
                        line_number: *ln,
                        line: text.clone().into_boxed_str(),
                        error: Cow::Borrowed("Invalid repeat count"),
                    })?;

                let start = i + 1;
                let mut depth = 1usize;
                let mut j = start;
                while j < block.len() {
                    let t = &block[j].1;
                    if t.starts_with("repeat ") {
                        depth += 1;
                    } else if t == "end" {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if depth != 0 {
                    return Err(TasError::ParsingError {
                        line_number: *ln,
                        line: text.clone().into_boxed_str(),
                        error: Cow::Borrowed("Unclosed repeat block"),
                    });
                }

                let nested = &block[start..j];
                apply_repeat_block(nested, n, state)?;
                i = j + 1;
            } else {
                let fb = parse_framebulk(state, *ln, text)?;
                state.framebulks.push(fb);
                i += 1;
            }
        }
    }
    Ok(())
}
