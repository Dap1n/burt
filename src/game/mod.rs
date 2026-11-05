pub mod cinput;
pub mod cvars;
pub mod engine_input;
pub mod entity;
pub mod hooks;
pub mod math;
pub mod modules;
pub mod offsets;
pub mod player;
pub mod signatures;

#[macro_export]
macro_rules! define_get_addr {
    ($static_name:ident, $fn_name:ident, $sig_name:ident, $module:expr $(,)?) => {
        static $static_name: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

        pub fn $fn_name() -> usize {
            *$static_name.get_or_init(|| $crate::game::modules::get_addr($module, $sig_name))
        }
    };
}
