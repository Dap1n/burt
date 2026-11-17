use crate::{
    commands::p2008::scale::ScaleStep,
    structs::{Color, Coordinates, Easing, TargetEntity},
};

pub trait BurtParam: Sized {
    fn from_input(input: &str) -> Result<Self, &'static str>;
}

impl BurtParam for String {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        Ok(input.to_string())
    }
}

impl BurtParam for bool {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        match input {
            "1" | "true" | "t" | "yes" | "y" | "enabled" => Ok(true),
            "0" | "false" | "f" | "no" | "n" | "disabled" => Ok(false),
            _ => Err("expects a parseable boolean"),
        }
    }
}

impl BurtParam for f32 {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let parsed: f32 = input
            .parse()
            .map_err(|_| "expects a parseable 32-bit float")?;
        if parsed.is_finite() {
            Ok(parsed)
        } else {
            Err("expects a finite 32-bit float")
        }
    }
}

impl BurtParam for i32 {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let input = input.to_lowercase();
        if let Some(stripped) = input.strip_prefix("0b") {
            Self::from_str_radix(stripped, 2)
        } else if let Some(stripped) = input.strip_prefix("0o") {
            Self::from_str_radix(stripped, 8)
        } else if let Some(stripped) = input.strip_prefix("0x") {
            Self::from_str_radix(stripped, 16)
        } else {
            input.parse()
        }
        .map_err(|_| "expects a parseable signed 32-bit integer")
    }
}

impl BurtParam for u32 {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let input = input.to_lowercase();
        if let Some(stripped) = input.strip_prefix("0b") {
            Self::from_str_radix(stripped, 2)
        } else if let Some(stripped) = input.strip_prefix("0o") {
            Self::from_str_radix(stripped, 8)
        } else if let Some(stripped) = input.strip_prefix("0x") {
            Self::from_str_radix(stripped, 16)
        } else {
            input.parse()
        }
        .map_err(|_| "expects a parseable unsigned 32-bit integer")
    }
}

impl BurtParam for Color {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let trimmed = input.trim();

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

        Err("expects a parseable color")
    }
}

impl BurtParam for Coordinates {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let normalized = input.replace(',', " ");
        let parts: Vec<_> = normalized.split_whitespace().collect();

        if parts.len() != 3 {
            return Err("expects exactly 3 coordinates (\"x y z\" or x,y,z)");
        }

        let mut coords = [0.0f32; 3];
        for (i, part) in parts.iter().enumerate() {
            let val: f32 = part.parse().map_err(|_| "invalid float format")?;

            if !val.is_finite() {
                return Err("expects finite numbers in coordinates");
            }

            coords[i] = val;
        }

        Ok(Coordinates {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        })
    }
}

impl BurtParam for Easing {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        Self::parse(input).ok_or("expects any of linear, sin, cubic, exp")
    }
}

impl BurtParam for TargetEntity {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        let input = input.trim();
        if input.is_empty() {
            return Err("expects an entity index or name");
        }

        if input.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(Self {
                name: None,
                index: input.parse().map_err(|_| "contains invalid entity index")?,
            });
        }

        let bracket_pos = input.find('[');
        let name = input[..bracket_pos.unwrap_or(input.len())].trim();

        if name.is_empty() {
            return Err("expects a non-empty entity name");
        }

        match bracket_pos {
            Some(pos) => Ok(Self {
                name: Some(name.into()),
                index: input[pos + 1..]
                    .strip_suffix(']')
                    .ok_or("is missing closing bracket")?
                    .trim()
                    .parse()
                    .map_err(|_| "contains malformed occurrence index")?,
            }),
            None => Ok(Self {
                name: Some(name.into()),
                index: 0,
            }),
        }
    }
}

impl BurtParam for ScaleStep {
    fn from_input(input: &str) -> Result<Self, &'static str> {
        if let Ok(step) = input.parse() {
            return Ok(Self::Exact(step));
        }
        match input {
            "min" => return Ok(Self::Min),
            "max" => return Ok(Self::Max),
            _ => (),
        }
        let Some(count) = input
            .strip_prefix("up")
            .or_else(|| input.strip_prefix("down"))
        else {
            return Err("expects a parseable signed 32-bit integer, min/max or up/down");
        };
        if count.is_empty() {
            if input.starts_with("up") {
                return Ok(Self::Relative(1));
            } else {
                return Ok(Self::Relative(-1));
            }
        }
        let Ok(n @ 1..) = count
            .trim_matches(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .parse()
        else {
            return Err("expects a parseable signed 32-bit non-zero integer after up/down");
        };
        if input.starts_with("up") {
            Ok(Self::Relative(n))
        } else {
            Ok(Self::Relative(-n))
        }
    }
}
