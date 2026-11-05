use crate::{clog, structs::Color};
use burt_command::burt_command;

/// Echoing text back to the console with custom color formatting.
///
/// `color:` which color to use:
///   - Named colors: "red", "light blue", "light-blue", "LIGHT_BLUE", etc.
///   - Hex with prefix: "#ff0000", "#f00", "#f00a", "0xff0000", "0Xf00"
///   - Bare hex (3/4/6/8 hex digits): "ff0000", "f00"
///   - Hex with internal whitespaces: "# ff 00 00"
/// `args:` special parameter that consumes everything afterwards
#[burt_command]
pub fn burt_echo(color: Color, args: String) {
    clog!([color => "{args}"]);
}
