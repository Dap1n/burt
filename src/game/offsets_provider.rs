use std::sync::OnceLock;

use crate::{
    engine_build::EngineBuild,
    game::offsets::{OffsetsProvider, j2009::July2009, p2008::Project2008},
};

static P2008_INSTANCE: Project2008 = Project2008;
static J2009_INSTANCE: July2009 = July2009;

static ACTIVE_PROVIDER: OnceLock<&'static dyn OffsetsProvider> = OnceLock::new();

type Initialized = bool;

pub fn init_active_build(engine_build: EngineBuild) -> Initialized {
    let provider: &dyn OffsetsProvider = match engine_build {
        EngineBuild::Project2008 => &P2008_INSTANCE,
        EngineBuild::July2009 => &J2009_INSTANCE,
        _ => return false,
    };
    ACTIVE_PROVIDER.set(provider).is_ok()
}

/// Helper to get the offset provider
#[inline(always)]
pub fn provider() -> &'static dyn OffsetsProvider {
    *ACTIVE_PROVIDER.get().expect("Build not detected")
}
