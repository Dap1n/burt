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
            [$crate::bridge::console::Color::LIME => "[BURT] "],
            $(
                [$color => $($arg)*],
            )+
        );
    };

    // Tagged prefix branch: clog!(prefix; "Plugin loading...", arg)
    (prefix; $($arg:tt)*) => {
        $crate::clog!(
            [$crate::bridge::console::Color::LIME => "[BURT] "],
            [$crate::bridge::console::Color::WHITE => $($arg)*],
        );
    };

    // Core multi-segment branch: clog!([col1 => "fmt", args...], [col2 => "fmt", args...])
    ($( [ $color:expr => $($arg:tt)* ] ),+ $(,)?) => {
        if let Some(&con_color_msg) = $crate::statics::CON_COLOR_MSG_FN.get() {
            $(
                $crate::bridge::console::con_color_msg(con_color_msg, &$color, &format!($($arg)*));
            )+
            $crate::bridge::console::con_color_msg(con_color_msg, &$crate::bridge::console::Color::WHITE, "\n");
        } else {
            $(
                $crate::log!("(Console unavailable) {}", format!($($arg)*));
            )+
        }
    };

    // Single-argument fallback: clog!("Plugin loading...", arg)
    ($($arg:tt)*) => {
        $crate::clog!(
            [$crate::bridge::console::Color::WHITE => $($arg)*]
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
        if !(-$lo..=$hi).contains(&$var) {
            $crate::clog!(
                [$crate::bridge::console::Color::RED => "{}", stringify!($var)],
                [$crate::bridge::console::Color::WHITE  => " parameter expects numbers to be in range "],
                [$crate::bridge::console::Color::RED => "[-{}; -{}]", $lo, $hi],
            );
            return;
        }
    };

    // Negative lower, non-negative upper:  -1 <= x <= 2
    (- $lo:literal <= $var:ident <= $hi:literal) => {
        if !(-$lo..=$hi).contains(&$var) {
            $crate::clog!(
                [$crate::bridge::console::Color::RED => "{}", stringify!($var)],
                [$crate::bridge::console::Color::WHITE  => " parameter expects numbers to be in range "],
                [$crate::bridge::console::Color::RED => "[-{}; {}]", $lo, $hi],
            );
            return;
        }
    };

    // Both bounds non-negative:  0 <= x <= 5
    ($lo:literal <= $var:ident <= $hi:literal) => {
        if !(-$lo..=$hi).contains(&$var) {
            $crate::clog!(
                [$crate::bridge::console::Color::RED => "{}", stringify!($var)],
                [$crate::bridge::console::Color::WHITE  => " parameter expects numbers to be in range "],
                [$crate::bridge::console::Color::RED => "[{}; {}]", $lo, $hi],
            );
            return;
        }
    };
}

/// Registers BURT commands
#[macro_export]
macro_rules! register_commands {
    ($( $name:expr => $help:expr => $callback:path ),* $(,)?) => {
        pub static PLUGIN_COMMANDS: [$crate::bridge::commands::SyncCommand; $crate::register_commands!(@count $($name),*)] = [
            $(
                $crate::bridge::commands::SyncCommand::new(
                    $crate::bridge::commands::ConCommand::new(
                        concat!($name, "\0"),
                        concat!($help, "\0"),
                        $callback,
                    )
                )
            ),*
        ];
    };

    (@count) => { 0usize };
    (@count $head:expr $(, $tail:expr)*) => { 1usize + $crate::register_commands!(@count $($tail),*) };
}
