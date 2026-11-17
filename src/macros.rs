use crate::engine_build::{EngineBuild, SUPPORTED_BUILDS_COUNT};

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_log {
    ($tag:expr, $($arg:tt)*) => {
        if let Some(filename) = $crate::statics::CRASH_FILENAME.get()
            && let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("crash_reports/{}.txt", filename))
        {
            use std::io::Write;
            let _ = writeln!(file, "[{}] {}", $tag, format_args!($($arg)*));
        }
    };
}

/// Writes into a file inside `crash_reports` folder
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::__internal_log!("BURT", $($arg)*);
    };
}

/// Writes error message into a file inside `crash_reports` folder
#[macro_export]
macro_rules! elog {
    ($($arg:tt)*) => {
        $crate::__internal_log!("ERR", $($arg)*);
    };
}

/// Writes debug message into a file inside `crash_reports` folder
#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => {
        $crate::__internal_log!(
            concat!(file!(), ":", line!(), ":", column!()),
            $($arg)*
        );
    };
}

/// Logs a colored message to the Source Engine console using `ConColorMsg`.
/// Falls back to file logging if the console interface is unavailable.
#[macro_export]
macro_rules! clog {
    // Tagged prefix branch: clog!(prefix; [ Color => "..." ], [ Color => "..." ])
    (prefix; $( [ $color:expr => $($arg:tt)* ] ),+ $(,)?) => {
        $crate::clog!(
            [$crate::structs::Color::LIME => "[BURT] "],
            $(
                [$color => $($arg)*],
            )+
        );
    };

    // Tagged error prefix branch: clog!(err_prefix; [ Color => "..." ], [ Color => "..." ])
    (err_prefix; $( [ $color:expr => $($arg:tt)* ] ),+ $(,)?) => {
        $crate::clog!(
            [$crate::structs::Color::RED => "[BURT] "],
            $(
                [$color => $($arg)*],
            )+
        );
    };

    // Tagged prefix branch: clog!(prefix; "Plugin loading...", arg)
    (prefix; $($arg:tt)*) => {
        $crate::clog!(
            [$crate::structs::Color::LIME => "[BURT] "],
            [$crate::structs::Color::WHITE => $($arg)*],
        );
    };

    // Tagged error prefix branch: clog!(err_prefix; "Plugin loading...", arg)
    (err_prefix; $($arg:tt)*) => {
        $crate::clog!(
            [$crate::structs::Color::RED => "[BURT] "],
            [$crate::structs::Color::WHITE => $($arg)*],
        );
    };

    // Core multi-segment branch: clog!([col1 => "fmt", args...], [col2 => "fmt", args...])
    ($( [ $color:expr => $($arg:tt)* ] ),+ $(,)?) => {
        if let Some(&con_color_msg) = $crate::statics::CON_COLOR_MSG_FN.get() {
            $(
                $crate::bridge::console::con_color_msg(con_color_msg, &$color, &format!($($arg)*));
            )+
            $crate::bridge::console::con_color_msg(con_color_msg, &$crate::structs::Color::WHITE, "\n");
        } else {
            $(
                $crate::log!("(Console unavailable) {}", format!($($arg)*));
            )+
        }
    };

    // Single-argument fallback: clog!("Plugin loading...", arg)
    ($($arg:tt)*) => {
        $crate::clog!(
            [$crate::structs::Color::WHITE => $($arg)*]
        );
    };
}

/// Executes a console command through the engine, formatting arguments
/// with `format!`.
#[macro_export]
macro_rules! cexec {
    ($($arg:tt)*) => {{
        let cmd = format!($($arg)*);
        if let Some(engine) = $crate::statics::ENGINE.get() {
            engine.execute(&cmd, true);
        } else {
            $crate::elog!("[cexec] ENGINE unavailable, dropped: {}", cmd);
        }
    }};
}

/// Bounds-check a variable and early-return if it's out of range.
///
/// On failure, logs a `clog!` message naming the variable and the
/// accepted range, then `return`s from the enclosing function.
#[macro_export]
macro_rules! define_bounds {
    // Both bounds negative:  -5 <= x <= -1
    (- $lo:literal <= $var:ident <= - $hi:literal) => {
        if !(-$lo..=-$hi).contains(&$var) {
            $crate::clog!(
                [$crate::structs::Color::RED => "{}", stringify!($var)],
                [$crate::structs::Color::WHITE => " parameter expects numbers to be in range "],
                [$crate::structs::Color::RED => "[-{}; -{}]", $lo, $hi],
            );
            return;
        }
    };

    // Negative lower, non-negative upper:  -1 <= x <= 2
    (- $lo:literal <= $var:ident <= $hi:literal) => {
        if !(-$lo..=$hi).contains(&$var) {
            $crate::clog!(
                [$crate::structs::Color::RED => "{}", stringify!($var)],
                [$crate::structs::Color::WHITE => " parameter expects numbers to be in range "],
                [$crate::structs::Color::RED => "[-{}; {}]", $lo, $hi],
            );
            return;
        }
    };

    // Both bounds non-negative:  0 <= x <= 5
    ($lo:literal <= $var:ident <= $hi:literal) => {
        if !($lo..=$hi).contains(&$var) {
            $crate::clog!(
                [$crate::structs::Color::RED => "{}", stringify!($var)],
                [$crate::structs::Color::WHITE => " parameter expects numbers to be in range "],
                [$crate::structs::Color::RED => "[{}; {}]", $lo, $hi],
            );
            return;
        }
    };
}

pub const fn extract_cmd_name(path: &'static str) -> &'static str {
    let bytes = path.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            start = i + 1;
        }
        i += 1;
    }
    path.split_at(start).1
}

pub const fn extract_builds(path: &'static str) -> [Option<EngineBuild>; SUPPORTED_BUILDS_COUNT] {
    let mut builds = [None; SUPPORTED_BUILDS_COUNT];
    let bytes = path.as_bytes();
    let mut builds_restricted = false;
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte != b':' && byte != b'_' {
            i += 1;
            continue;
        }
        if start < i {
            let s = bytes.split_at(start).1.split_at(i - start).0;
            if let Some(build) = EngineBuild::from_shorthand(s) {
                builds[build as usize] = Some(build);
                builds_restricted = true;
            }
        }
        start = i + 1;
        i += 1;
    }
    if !builds_restricted {
        i = 0;
        while i < SUPPORTED_BUILDS_COUNT {
            builds[i] = Some(EngineBuild::from_index(i as u8));
            i += 1;
        }
    }
    builds
}

/// Registers BURT commands
#[macro_export]
macro_rules! register_commands {
    ($( $path:path $(| $autocomplete:path)? => $help:expr ),* $(,)?) => {
        pub static PLUGIN_COMMANDS: [
            ($crate::bridge::commands::SyncCommand, [Option<$crate::engine_build::EngineBuild>; $crate::engine_build::SUPPORTED_BUILDS_COUNT]);
            $crate::register_commands!(@count $($path),*)
        ] = [
            $(
                (
                    $crate::bridge::commands::SyncCommand::new(
                        $crate::bridge::commands::ConCommand::new(
                            $crate::macros::extract_cmd_name(concat!(stringify!($path), "\0")),
                            concat!($help, "\0"),
                            $path,
                            $crate::register_commands!(@opt_auto $($autocomplete)?)
                        )
                    ),
                    $crate::macros::extract_builds(stringify!($path)),
                )
            ),*
        ];
    };

    (@opt_auto) => { None };
    (@opt_auto $autocomplete:path) => { Some($autocomplete) };

    (@count) => { 0usize };
    (@count $head:path $(, $tail:path)*) => { 1usize + $crate::register_commands!(@count $($tail),*) };
}
