use std::{
    ffi::CString,
    fmt, mem,
    os::raw::{c_char, c_void},
    str::FromStr,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // Basic colors
    pub const RED: Self = Self::rgb(255, 64, 64);
    pub const GREEN: Self = Self::rgb(64, 255, 64);
    pub const BLUE: Self = Self::rgb(64, 150, 255);
    pub const YELLOW: Self = Self::rgb(255, 220, 64);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const GRAY: Self = Self::rgb(160, 160, 160);

    // Extended basics
    pub const ORANGE: Self = Self::rgb(255, 165, 64);
    pub const PURPLE: Self = Self::rgb(180, 100, 255);
    pub const PINK: Self = Self::rgb(255, 130, 190);
    pub const CYAN: Self = Self::rgb(64, 230, 230);
    pub const MAGENTA: Self = Self::rgb(255, 80, 200);
    pub const LIME: Self = Self::rgb(150, 255, 80);
    pub const TEAL: Self = Self::rgb(64, 200, 180);
    pub const BROWN: Self = Self::rgb(150, 100, 60);

    // Shades
    pub const LIGHT_GRAY: Self = Self::rgb(210, 210, 210);
    pub const DARK_GRAY: Self = Self::rgb(80, 80, 80);
    pub const LIGHT_RED: Self = Self::rgb(255, 140, 140);
    pub const DARK_RED: Self = Self::rgb(180, 30, 30);
    pub const LIGHT_GREEN: Self = Self::rgb(140, 255, 140);
    pub const DARK_GREEN: Self = Self::rgb(30, 150, 30);
    pub const LIGHT_BLUE: Self = Self::rgb(140, 200, 255);
    pub const DARK_BLUE: Self = Self::rgb(30, 80, 180);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseColorError {
    UnknownColor(String),
    InvalidHex(String),
}

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownColor(s) => write!(f, "unknown color name: {s:?}"),
            Self::InvalidHex(s) => write!(f, "invalid hex color: {s:?}"),
        }
    }
}

impl std::error::Error for ParseColorError {}

impl Color {
    /// Parse a color from a string.
    ///
    /// Accepts:
    ///   - Named colors: "red", "light blue", "light-blue", "LIGHT_BLUE", ...
    ///   - Hex with prefix: "#ff0000", "#f00", "#f00a", "0xff0000", "0Xf00"
    ///   - Bare hex (3/4/6/8 hex digits): "ff0000", "f00"
    ///   - Internal whitespace is ignored in hex: "# ff 00 00"
    pub fn parse(s: &str) -> Result<Self, ParseColorError> {
        let trimmed = s.trim();

        let hex_body = trimmed
            .strip_prefix('#')
            .or_else(|| trimmed.strip_prefix("0x"))
            .or_else(|| trimmed.strip_prefix("0X"));

        if let Some(body) = hex_body {
            return Self::from_hex(body);
        }

        let normalized = trimmed.to_ascii_lowercase().replace([' ', '-'], "_");

        let named = match normalized.as_str() {
            "red" => Some(Self::RED),
            "green" => Some(Self::GREEN),
            "blue" => Some(Self::BLUE),
            "yellow" => Some(Self::YELLOW),
            "white" => Some(Self::WHITE),
            "black" => Some(Self::BLACK),
            "gray" | "grey" => Some(Self::GRAY),

            "orange" => Some(Self::ORANGE),
            "purple" => Some(Self::PURPLE),
            "pink" => Some(Self::PINK),
            "cyan" => Some(Self::CYAN),
            "magenta" => Some(Self::MAGENTA),
            "lime" => Some(Self::LIME),
            "teal" => Some(Self::TEAL),
            "brown" => Some(Self::BROWN),

            "light_gray" | "light_grey" => Some(Self::LIGHT_GRAY),
            "dark_gray" | "dark_grey" => Some(Self::DARK_GRAY),
            "light_red" => Some(Self::LIGHT_RED),
            "dark_red" => Some(Self::DARK_RED),
            "light_green" => Some(Self::LIGHT_GREEN),
            "dark_green" => Some(Self::DARK_GREEN),
            "light_blue" => Some(Self::LIGHT_BLUE),
            "dark_blue" => Some(Self::DARK_BLUE),

            _ => None,
        };

        if let Some(c) = named {
            return Ok(c);
        }

        let compact: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if matches!(compact.len(), 3 | 4 | 6 | 8) && compact.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Self::from_hex(&compact);
        }

        Err(ParseColorError::UnknownColor(trimmed.to_string()))
    }

    fn from_hex(raw: &str) -> Result<Self, ParseColorError> {
        let hex: String = raw.chars().filter(|c| !c.is_whitespace()).collect();

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseColorError::InvalidHex(raw.to_string()));
        }

        let nibble = |i: usize| -> u8 { u8::from_str_radix(&hex[i..i + 1], 16).unwrap() };
        let byte = |i: usize| -> u8 { u8::from_str_radix(&hex[i..i + 2], 16).unwrap() };

        match hex.len() {
            3 => Ok(Self::rgb(nibble(0) * 17, nibble(1) * 17, nibble(2) * 17)),
            4 => Ok(Self::rgba(
                nibble(0) * 17,
                nibble(1) * 17,
                nibble(2) * 17,
                nibble(3) * 17,
            )),
            6 => Ok(Self::rgb(byte(0), byte(2), byte(4))),
            8 => Ok(Self::rgba(byte(0), byte(2), byte(4), byte(6))),
            _ => Err(ParseColorError::InvalidHex(raw.to_string())),
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

pub type ConColorMsgFn = unsafe extern "C" fn(color: *const Color, fmt: *const c_char, ...);

pub fn get_con_color_msg() -> Option<ConColorMsgFn> {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const c_char) -> *mut c_void;
        fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    }

    let module_name = CString::new("tier0.dll").ok()?;
    let func_name = CString::new("?ConColorMsg@@YAXABVColor@@PBDZZ").ok()?;

    unsafe {
        let module = GetModuleHandleA(module_name.as_ptr());
        if !module.is_null() {
            let func = GetProcAddress(module, func_name.as_ptr());
            if !func.is_null() {
                return Some(mem::transmute::<*mut c_void, ConColorMsgFn>(func));
            }
        }
    }

    None
}

pub fn con_color_msg(func: ConColorMsgFn, color: &Color, msg: &str) {
    let c_str = match CString::new(msg) {
        Ok(s) => s,
        Err(_) => CString::new(msg.replace('\0', "\\0")).unwrap(),
    };

    unsafe {
        func(color, c"%s".as_ptr() as *const c_char, c_str.as_ptr());
    }
}
