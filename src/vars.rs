use std::sync::atomic::*;

use portable_atomic::{AtomicF32, AtomicF64};

pub trait BurtVar {
    fn display(&self) -> String;
}

impl BurtVar for AtomicU32 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicI32 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicF32 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicU64 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicI64 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicF64 {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
impl BurtVar for AtomicBool {
    fn display(&self) -> String {
        self.load(Ordering::Relaxed).to_string()
    }
}
