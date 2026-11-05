use crate::{cexec, clog};
use burt_command::burt_command;

/// Safely unloads the plugin.
///
/// It's basically a wrapper around `plugin_unload burt`,
/// since BURT handles hooks and commands unloading automatically.
#[burt_command]
pub fn burt_exit() {
    clog!(prefix; "Unloading the plugin...");
    cexec!("plugin_unload burt");
}
