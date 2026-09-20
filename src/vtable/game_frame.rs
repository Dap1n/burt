use std::{
    ffi::CStr,
    mem,
    os::raw::{c_char, c_void},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery,
};

use crate::{
    bridge::invoke::{edict_t, get_edict_of_ent_index},
    commands::{
        self,
        rec::{CURRENT_MAP, IN_LOAD, on_map_ready},
        run_state,
    },
    elog,
};

const EDICT_ENTITY_OFF: usize = 0x0C;
const M_I_NAME_OFF: usize = 0x120;
const LAST_MAP_NAME: &str = "p2_lab_hub_6";
const COMPLETION_ENTITY_NAME: &str = "campaign_incomplete_meow";

fn readable(addr: usize, len: usize) -> bool {
    if addr < 0x10000 || addr >= 0x8000_0000 {
        return false;
    }
    unsafe {
        let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();
        let ret = VirtualQuery(
            addr as *const c_void,
            &mut mbi,
            mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        if ret == 0
            || mbi.State != MEM_COMMIT
            || mbi.Protect & PAGE_GUARD != 0
            || mbi.Protect & PAGE_NOACCESS != 0
        {
            return false;
        }
        addr + len <= (mbi.BaseAddress as usize) + mbi.RegionSize
    }
}

unsafe fn entity_from_edict(edict: *const edict_t) -> *const u8 {
    unsafe { *((edict as *const c_void as *const u8).add(EDICT_ENTITY_OFF) as *const *const u8) }
}

unsafe fn read_name(entity: *const u8) -> Option<String> {
    let ptr = unsafe { *(entity.add(M_I_NAME_OFF) as *const *const c_char) };
    if ptr.is_null() {
        return None;
    }
    let addr = ptr as usize;
    if !readable(addr, 2) {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

unsafe fn campaign_incomplete_data() -> Option<(*mut edict_t, *const u8)> {
    for i in 1..2048 {
        unsafe {
            let edict = get_edict_of_ent_index(i);
            if edict.is_null() {
                continue;
            }
            let entity = entity_from_edict(edict);
            if entity.is_null() {
                continue;
            }
            if read_name(entity).as_deref() == Some(COMPLETION_ENTITY_NAME) {
                return Some((edict, entity));
            }
        }
    }
    None
}

pub fn game_frame() {
    if !run_state::is_recording() {
        return;
    }

    if IN_LOAD.swap(false, Ordering::Relaxed) {
        let map = CURRENT_MAP.read().unwrap().clone();

        if run_state::take_no_vault()
            && let Ok(mut enigo) = Enigo::new(&Settings::default())
        {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(700));
                for _ in 0..8 {
                    thread::sleep(Duration::from_millis(500));
                    let _ = enigo.key(Key::Return, Press);
                    let _ = enigo.key(Key::Return, Release);
                }
            });
        }

        on_map_ready(&map);

        if map != LAST_MAP_NAME {
            run_state::clear_marker();
            return;
        }

        let Some((edict, entity)) = (unsafe { campaign_incomplete_data() }) else {
            elog!("Completion entity not found");
            return;
        };
        run_state::set_marker(edict as usize, entity as usize);
        return;
    }

    let Some((edict, entity)) = run_state::get_marker() else {
        return;
    };

    if unsafe { entity_from_edict(edict as *const edict_t) } != entity as *const u8 {
        commands::sr_stop::stop_internal(true);
    }
}
