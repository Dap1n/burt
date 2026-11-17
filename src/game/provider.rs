use std::sync::OnceLock;

use crate::{
    engine_build::EngineBuild,
    game::{July2009, Project2008, metadata::MetadataProvider, offsets::OffsetsProvider},
};

static P2008_INSTANCE: Project2008 = Project2008;
static J2009_INSTANCE: July2009 = July2009;

pub trait Provider: OffsetsProvider + MetadataProvider {}
impl Provider for July2009 {}
impl Provider for Project2008 {}

static ACTIVE_PROVIDER: OnceLock<&'static dyn Provider> = OnceLock::new();

pub fn init_active_build(engine_build: EngineBuild) -> Result<(), &'static str> {
    let provider: &dyn Provider = match engine_build {
        EngineBuild::Project2008 => &P2008_INSTANCE,
        EngineBuild::July2009 => &J2009_INSTANCE,
        _ => return Err("This build is not supported yet"),
    };
    ACTIVE_PROVIDER
        .set(provider)
        .map_err(|_| "Failed to set ACTIVE_PROVIDER")
}

/// Helper to get the offset provider
#[inline(always)]
pub fn provider() -> &'static dyn Provider {
    *ACTIVE_PROVIDER.get().expect("Build not detected")
}
