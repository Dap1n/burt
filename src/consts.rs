use std::ffi::CStr;

pub const DESC: &CStr = match CStr::from_bytes_until_nul(
    concat!(
        "BURT (",
        env!("CARGO_PKG_DESCRIPTION"),
        ") v",
        env!("CARGO_PKG_VERSION"),
        "\0",
    )
    .as_bytes(),
) {
    Ok(s) => s,
    Err(_) => panic!("invalid CStr literal for DESC"),
};

pub const CRASH_FORMAT: &str = "%Y-%m-%d_%H.%M.%S";
pub const RECORDS_FORMAT: &str = "%Y%m%d_%H%M%S_%6f";
pub const SPEEDRUN_FOLDER_FORMAT: &str = "%Y-%m-%d_%H.%M.%S";
