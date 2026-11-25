use crate::{
    bridge::{commands::CCommand, console::con_color_msg},
    params::BurtParam,
    statics::CON_COLOR_MSG_FN,
    structs::Color,
};

pub struct LogSegment<'a> {
    pub color: Color,
    pub text: &'a str,
}

pub struct CommandInfo {
    pub prefix: &'static [LogSegment<'static>],
    pub doc: &'static [LogSegment<'static>],
    pub var_display: Option<fn() -> String>,
    pub required_params_exist: bool,
    pub arg_index: usize,
}

#[inline]
fn log_message(info: &CommandInfo, tail: &[LogSegment]) {
    let Some(&func) = CON_COLOR_MSG_FN.get() else {
        return;
    };
    for seg in info.prefix {
        con_color_msg(func, &seg.color, seg.text);
    }
    if let Some(display) = info.var_display {
        let text = display();
        con_color_msg(func, &Color::YELLOW, "Current value: ");
        con_color_msg(func, &Color::WHITE, &text);
    }
    for seg in tail {
        con_color_msg(func, &seg.color, seg.text);
    }
    con_color_msg(func, &Color::WHITE, "\n");
}

#[inline(never)]
pub fn check_help(command: &CCommand, info: &CommandInfo) -> bool {
    if info.required_params_exist && command.is_empty()
        || command
            .get_args_remainder(1)
            .is_some_and(|s| s.trim_end() == "?")
    {
        log_message(info, info.doc);
        true
    } else {
        false
    }
}

#[inline(never)]
pub fn check_extra_args(command: &CCommand, info: &CommandInfo) -> bool {
    if command.get_arg(info.arg_index).is_some() {
        log_message(
            info,
            &[LogSegment {
                color: Color::RED,
                text: "Too many arguments",
            }],
        );
        true
    } else {
        false
    }
}

#[inline(never)]
pub fn parse_param<T: BurtParam>(
    command: &CCommand,
    info: &CommandInfo,
    index: usize,
    name: &str,
    required: bool,
    special: bool,
) -> Result<Option<T>, ()> {
    let input = if special {
        command.get_args_remainder(index)
    } else {
        command.get_arg(index)
    };
    let label = if special {
        "Special parameter "
    } else {
        "Parameter "
    };

    if let Some(input) = input {
        if input == "_" && !required {
            return Ok(None);
        }
        match <T as BurtParam>::from_input(input) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(err) => {
                log_message(
                    info,
                    &[
                        LogSegment {
                            color: Color::WHITE,
                            text: label,
                        },
                        LogSegment {
                            color: Color::RED,
                            text: name,
                        },
                        LogSegment {
                            color: Color::WHITE,
                            text: " ",
                        },
                        LogSegment {
                            color: Color::WHITE,
                            text: err,
                        },
                    ],
                );
                Err(())
            }
        }
    } else if required {
        log_message(
            info,
            &[
                LogSegment {
                    color: Color::WHITE,
                    text: label,
                },
                LogSegment {
                    color: Color::RED,
                    text: name,
                },
                LogSegment {
                    color: Color::WHITE,
                    text: " is required but not provided",
                },
            ],
        );
        Err(())
    } else {
        Ok(None)
    }
}
