use crate::{bridge::pe, game::modules};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineBuild {
    July2009,
    February2010,
    March2010,
    July2010,
    August2010,
    October2010,
}

impl EngineBuild {
    pub fn get_current() -> Option<Self> {
        match pe::module_timestamp(modules::engine()) {
            Some(1248818797) => Some(Self::July2009),
            Some(1265294906) => Some(Self::February2010),
            Some(1268957049) => Some(Self::March2010),
            Some(1279837492) => Some(Self::July2010),
            Some(1283206058) => Some(Self::August2010),
            Some(1287508065) => Some(Self::October2010),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::July2009 => "July 2009",
            Self::February2010 => "February 2010",
            Self::March2010 => "March 2010",
            Self::July2010 => "July 2010",
            Self::August2010 => "August 2010",
            Self::October2010 => "October 2010",
        }
    }

    pub fn number(&self) -> &'static str {
        match self {
            Self::July2009 => "852_0",
            Self::February2010 => "841_0",
            Self::March2010 => "852_1",
            Self::July2010 => "852_2",
            Self::August2010 => "852_3",
            Self::October2010 => "852_4",
        }
    }
}
