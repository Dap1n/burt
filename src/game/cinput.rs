use std::sync::atomic::{AtomicU8, Ordering};

use crate::{
    commands::tas::HeldButtons,
    game::{modules, provider::provider, signatures},
};

pub static HELD_BUTTONS: AtomicU8 = AtomicU8::new(0);

#[derive(Copy, Clone)]
pub enum Action {
    Attack,
    Duck,
    Jump,
    Use,
    Attack2,
    Slowtime,
}

impl Action {
    #[inline]
    pub const fn bitmask(self) -> u8 {
        HeldButtons::get_n(self)
    }

    #[inline]
    pub fn is_held(&self) -> bool {
        (HELD_BUTTONS.load(Ordering::Relaxed) & self.bitmask()) != 0
    }

    fn update_held(&self, press: bool) {
        let mask = self.bitmask();
        let mut current = HELD_BUTTONS.load(Ordering::Relaxed);
        let is_currently_held = (current & mask) != 0;

        if is_currently_held != press {
            if press {
                current |= mask;
            } else {
                current &= !mask;
            }
            HELD_BUTTONS.store(current, Ordering::Relaxed);
        }
    }

    #[inline]
    unsafe fn get_addr(&self, slot: usize) -> usize {
        let func_addr = modules::get_addr(modules::client(), provider().get_button_bits_sig());
        let (offset, extra) = provider().action_offset(*self);
        let array_base = unsafe { signatures::read_abs_addr(func_addr + offset) };
        array_base + extra + slot * provider().button_state_stride()
    }

    pub fn press(&self, slot: usize) {
        self.update_held(true);
        unsafe {
            let addr = self.get_addr(slot);
            *(addr as *mut u32) = 5;
        }
    }

    pub fn release(&self, slot: usize) {
        self.update_held(false);
        unsafe {
            let addr = self.get_addr(slot);
            *(addr as *mut u32) = 4;
        }
    }
}

pub unsafe fn inject_mouse_delta(cinput: *mut u8, dx: f32, dy: f32) {
    unsafe {
        let base = cinput.add(
            provider().cinput_slot() * provider().cinput_slot_stride()
                + provider().cinput_mouse_off_in_slot(),
        );
        *(base as *mut f32) = dx;
        *(base.add(4) as *mut f32) = dy;
    }
}
