use crate::game::July2009;

use super::MetadataProvider;

impl MetadataProvider for July2009 {
    fn main_folders(&self) -> &'static [&'static str] {
        &["portal2"]
    }
    fn tas_extension(&self) -> &'static str {
        ".p2btas"
    }
    fn first_map(&self) -> &'static str {
        "p2_lab_prehub_1"
    }
    fn last_map(&self) -> &'static str {
        "p2_lab_hub_6"
    }
    fn tickrate(&self) -> f32 {
        60.0
    }
    fn move_speed(&self) -> f32 {
        175.0
    }
}
