use std::sync::OnceLock;

use crate::{bridge::pe, game::modules};

pub const SUPPORTED_BUILDS_COUNT: usize = 2;
static CURRENT_BUILD: OnceLock<u8> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineBuild {
    Project2008 = 0,
    July2009 = 1,
    February2010 = 2,
    March2010 = 3,
    July2010 = 4,
    August2010 = 5,
    October2010 = 6,
}

impl EngineBuild {
    pub fn get_current() -> Option<Self> {
        match pe::module_timestamp(modules::engine()) {
            Some(1596012100) => Some(Self::Project2008),
            Some(1248818797) => Some(Self::July2009),
            Some(1265294906) => Some(Self::February2010),
            Some(1268957049) => Some(Self::March2010),
            Some(1279837492) => Some(Self::July2010),
            Some(1283206058) => Some(Self::August2010),
            Some(1287508065) => Some(Self::October2010),
            _ => None,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Project2008 => "Project 2008",
            Self::July2009 => "July 2009",
            Self::February2010 => "February 2010",
            Self::March2010 => "March 2010",
            Self::July2010 => "July 2010",
            Self::August2010 => "August 2010",
            Self::October2010 => "October 2010",
        }
    }

    pub const fn tag(&self) -> &'static str {
        match self {
            Self::Project2008 => "F-Stop",
            Self::July2009 => "852_0",
            Self::February2010 => "841_0",
            Self::March2010 => "852_1",
            Self::July2010 => "852_2",
            Self::August2010 => "852_3",
            Self::October2010 => "852_4",
        }
    }

    pub const fn from_shorthand(s_utf8: &[u8]) -> Option<Self> {
        match s_utf8 {
            b"p2008" => Some(Self::Project2008),
            b"j2009" => Some(Self::July2009),
            b"f2010" => Some(Self::February2010),
            b"m2010" => Some(Self::March2010),
            b"j2010" => Some(Self::July2010),
            b"a2010" => Some(Self::August2010),
            b"o2010" => Some(Self::October2010),
            _ => None,
        }
    }

    pub const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Project2008,
            1 => Self::July2009,
            2 => Self::February2010,
            3 => Self::March2010,
            4 => Self::July2010,
            5 => Self::August2010,
            6 => Self::October2010,
            _ => unreachable!(),
        }
    }
}

pub fn save(build: EngineBuild) {
    CURRENT_BUILD
        .set(build as u8)
        .expect("Build can only be set once");
}

pub fn current_build() -> EngineBuild {
    EngineBuild::from_index(*CURRENT_BUILD.get().expect("Build not initialized"))
}
