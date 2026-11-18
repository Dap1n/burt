use crate::game::{Project2008, math::Vec3};

use super::MetadataProvider;

impl MetadataProvider for Project2008 {
    fn main_folder(&self) -> &'static str {
        "p2008"
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
    fn in_end_position(&self, view: Vec3) -> bool {
        (-2980.0..=-2840.0).contains(&view.0)
            && (-2310.0..=-2280.0).contains(&view.1)
            && (-3072.0..=-2950.0).contains(&view.2)
    }
    fn tickrate(&self) -> f32 {
        66.0 + 2.0 / 3.0
    }
    fn move_speed(&self) -> f32 {
        275.0
    }
}
