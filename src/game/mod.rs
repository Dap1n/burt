pub mod cinput;
pub mod cvars;
pub mod engine_input;
pub mod entity;
pub mod hooks;
pub mod math;
pub mod metadata;
pub mod modules;
pub mod offsets;
pub mod player;
pub mod provider;
pub mod signatures;

/// Zero-sized struct representing the Portal 2 July 2009 Beta build.
pub struct July2009;

/// Zero-sized struct representing the F-Stop recreation in Project 2008.
pub struct Project2008;

#[macro_export]
macro_rules! define_get_addr {
    ($static_name:ident, $fn_name:ident, $sig_name:ident, $module:expr $(,)?) => {
        static $static_name: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

        pub fn $fn_name() -> usize {
            *$static_name.get_or_init(|| {
                $crate::game::modules::get_addr(
                    $module,
                    $crate::game::provider::provider().$sig_name(),
                )
            })
        }
    };
}
