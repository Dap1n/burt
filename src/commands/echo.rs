use std::str::FromStr;

use crate::{
    bridge::{commands::CCommand, console::Color},
    clog,
};
use burt_macros::burt_command;

/// Echoing text back to console with custom color formatting.
/// `color` parameter accepts:
///   - Named colors: "red", "light blue", "light-blue", "LIGHT_BLUE", ...
///   - Hex with prefix: "#ff0000", "#f00", "#f00a", "0xff0000", "0Xf00"
///   - Bare hex (3/4/6/8 hex digits): "ff0000", "f00"
///   - Hex with internal whitespaces: "# ff 00 00"
#[burt_command]
pub fn burt_echo(color: String, args: String) {
    let Ok(color) = Color::from_str(&color) else {
        clog!(prefix; "Invalid color {color:?}");
        return;
    };
    clog!([color => "{}", args]);
}
