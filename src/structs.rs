use core::fmt;
use std::f32::consts::FRAC_PI_2;

use crate::game::entity::{self, edict_t};

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

impl Color {
    pub fn from_hex(raw: &str) -> Result<Self, &'static str> {
        let hex: String = raw.chars().filter(|c| !c.is_whitespace()).collect();

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("expects a valid HEX");
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
            _ => Err("expects a valid HEX"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}, {:.2}, {:.2}", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetEntity {
    pub name: Option<Box<str>>,
    pub index: u32,
}

impl TargetEntity {
    pub fn fetch(&self) -> Option<(*mut edict_t, *const u8)> {
        if let Some(name) = self.name.as_deref() {
            unsafe { entity::edict_entity_by_name(name, self.index as i32) }
        } else {
            unsafe { entity::edict_entity_by_index(self.index as i32) }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Easing {
    #[default]
    Linear = 0,
    Sine = 1,
    Cubic = 2,
    Exponential = 3,
}

impl Easing {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "linear" => Some(Self::Linear),
            "sin" | "sine" => Some(Self::Sine),
            "cubic" => Some(Self::Cubic),
            "exp" | "exponential" => Some(Self::Exponential),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self as u8
    }

    pub const fn from_index(i: u8) -> Self {
        match i {
            1 => Self::Sine,
            2 => Self::Cubic,
            3 => Self::Exponential,
            _ => Self::Linear,
        }
    }

    pub fn apply(self, progress: f32) -> f32 {
        if progress >= 1.0 {
            return 1.0;
        }
        match self {
            Self::Linear => progress,
            Self::Sine => (progress * FRAC_PI_2).sin(),
            Self::Cubic => progress.powi(3),
            Self::Exponential => {
                if progress <= 0.0 {
                    0.0
                } else if progress >= 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * progress)
                }
            }
        }
    }
}
