use crate::game::Project2008;

use super::MetadataProvider;

impl MetadataProvider for Project2008 {
    fn main_folders(&self) -> &'static [&'static str] {
        &["aperture", "p2008"]
    }
    fn tas_extension(&self) -> &'static str {
        ".fstas"
    }
    fn first_map(&self) -> &'static str {
        "lab_intro"
    }
    fn last_map(&self) -> &'static str {
        "lab_fan_lift"
    }
    fn tickrate(&self) -> f32 {
        66.0 + 2.0 / 3.0
    }
    fn move_speed(&self) -> f32 {
        275.0
    }
}
