use std::cmp::Ordering;

pub mod accumulate_time;
pub mod create_move;
pub mod input_filter;
pub mod physics;
pub mod process_cmd;
pub mod render_view;

pub fn install_all() {
    unsafe {
        accumulate_time::install();
        create_move::install();
        input_filter::install();
        physics::install();
        process_cmd::install();
        render_view::install();
    }
}

pub fn uninstall_all() {
    unsafe {
        accumulate_time::uninstall();
        create_move::uninstall();
        input_filter::uninstall();
        physics::uninstall();
        process_cmd::uninstall();
        render_view::uninstall();
    }
}

type FnRef<T> = fn(&mut T);

#[derive(Debug, Clone, Copy)]
pub enum Subscriber<T> {
    Setang(FnRef<T>),
    Autoaim(FnRef<T>),
    Autojump(FnRef<T>),
    Absmov(FnRef<T>),
    Strafe(FnRef<T>),
    Decel(FnRef<T>),
    SgDbg(FnRef<T>),
    Speedrun(FnRef<T>),
}

impl<T> PartialEq for Subscriber<T> {
    fn eq(&self, other: &Self) -> bool {
        // compared by tag for dedup
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl<T> Eq for Subscriber<T> {}

impl<T> PartialOrd for Subscriber<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Subscriber<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |cmd| match cmd {
            &Self::Setang(_) => 0,
            Self::Autoaim(_) => 1,
            Self::Autojump(_) => 2,
            Self::Absmov(_) => 3,
            Self::Strafe(_) => 4,
            Self::Decel(_) => 5,
            Self::SgDbg(_) => 6,
            Self::Speedrun(_) => 7,
        };
        rank(self).cmp(&rank(other))
    }
}

impl<T> Subscriber<T> {
    pub fn get_fn(&self) -> FnRef<T> {
        match self {
            Self::Setang(f) => *f,
            Self::Autoaim(f) => *f,
            Self::Autojump(f) => *f,
            Self::Absmov(f) => *f,
            Self::Strafe(f) => *f,
            Self::Decel(f) => *f,
            Self::SgDbg(f) => *f,
            Self::Speedrun(f) => *f,
        }
    }
}

#[macro_export]
macro_rules! define_subscriptions {
    ($ty:ty) => {
        pub static SUBS: ::std::sync::Mutex<Vec<$crate::game::hooks::Subscriber<$ty>>> =
            ::std::sync::Mutex::new(Vec::new());

        pub fn subscribe(cmd: $crate::game::hooks::Subscriber<$ty>) {
            let mut subs = SUBS.lock().unwrap();
            if !subs.contains(&cmd) {
                let idx = subs.binary_search(&cmd).unwrap_or_else(|e| e);
                subs.insert(idx, cmd);
            }
        }

        pub fn unsubscribe(cmd: $crate::game::hooks::Subscriber<$ty>) {
            SUBS.lock().unwrap().retain(|x| x != &cmd);
        }
    };
}
