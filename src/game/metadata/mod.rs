use crate::game::math::Vec3;

pub mod j2009;
pub mod p2008;

/// The interface defining metadata for different builds.
pub trait MetadataProvider: Send + Sync {
    fn main_folder(&self) -> &'static str;
    fn tas_extension(&self) -> &'static str;
    fn first_map(&self) -> &'static str;
    fn last_map(&self) -> &'static str;
    fn in_end_position(&self, view: Vec3) -> bool;
    fn move_speed(&self) -> f32;
}
