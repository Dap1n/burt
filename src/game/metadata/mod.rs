pub mod j2009;
pub mod p2008;

/// The interface defining metadata for different builds.
pub trait MetadataProvider: Send + Sync {
    fn main_folders(&self) -> &'static [&'static str];
    fn tas_extension(&self) -> &'static str;
    fn first_map(&self) -> &'static str;
    fn last_map(&self) -> &'static str;
    fn tickrate(&self) -> f32;
    fn move_speed(&self) -> f32;
}
