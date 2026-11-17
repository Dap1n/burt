use std::{
    cell::UnsafeCell,
    ffi::{CStr, c_char, c_int, c_void},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{clog, elog, engine_build, log};

pub type CreateInterfaceFn =
    unsafe extern "cdecl" fn(p_name: *const c_char, p_return_code: *mut c_int) -> *mut c_void;

pub static ICVAR: AtomicUsize = AtomicUsize::new(0);

const ICVAR_REGISTER_COMMAND_SLOT: usize = 9;
const ICVAR_UNREGISTER_COMMAND_SLOT: usize = 10;

#[repr(C)]
pub struct CCommand {
    argc: c_int,
    _argv0_size: c_int,
    arg_s_buffer: [c_char; 512],
    arg_buffer: [c_char; 512],
    args: [*const c_char; 64],
}

impl CCommand {
    pub fn get_arg(&self, index: usize) -> Option<&str> {
        if index >= self.argc as usize || index >= 64 {
            return None;
        }
        let ptr = self.args[index];
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Returns whole raw command line as written by the player.
    pub fn get_line(&self) -> &CStr {
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.arg_s_buffer.as_ptr() as *const u8,
                self.arg_s_buffer.len(),
            )
        };
        CStr::from_bytes_until_nul(bytes).unwrap_or(c"")
    }

    /// Total number of arguments (including command name)
    #[inline]
    pub const fn len(&self) -> usize {
        self.argc as usize
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.argc <= 1
    }

    /// Returns raw remaining command line starting from argument index `arg_index`
    pub fn get_args_remainder(&self, arg_index: usize) -> Option<&str> {
        if arg_index >= self.len() {
            return None;
        }

        let mut curr = self.get_line().to_str().ok()?.trim_start();

        for _ in 0..arg_index {
            if curr.is_empty() {
                return None;
            }

            if curr.starts_with('"') {
                let rest = &curr[1..];
                let mut end_idx = None;
                let mut escaped = false;

                for (i, b) in rest.bytes().enumerate() {
                    if escaped {
                        escaped = false;
                    } else if b == b'\\' {
                        escaped = true;
                    } else if b == b'"' {
                        end_idx = Some(i);
                        break;
                    }
                }

                if let Some(quote_pos) = end_idx {
                    curr = &curr[1 + quote_pos + 1..];
                } else {
                    curr = "";
                }
            } else if let Some(ws_idx) = curr.find(|c: char| c.is_whitespace()) {
                curr = &curr[ws_idx..];
            } else {
                curr = "";
            }

            curr = curr.trim_start();
        }

        (!curr.is_empty()).then_some(curr)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AutocompleteError {
    TooManyCommands,
    InputTooLong,
}

#[derive(Debug, Clone)]
pub struct AutocompleteWriter {
    buffer: *mut c_char,
    count: usize,
}

impl AutocompleteWriter {
    #[inline]
    pub const unsafe fn new(buffer: *mut c_char) -> Self {
        Self { buffer, count: 0 }
    }

    pub fn push(&mut self, s: &str) -> Result<(), AutocompleteError> {
        if self.count >= 64 {
            return Err(AutocompleteError::TooManyCommands);
        }

        let bytes = s.as_bytes();
        let len = bytes.len();
        if len >= 64 {
            return Err(AutocompleteError::InputTooLong);
        }

        unsafe {
            let dest = self.buffer.add(self.count * 64);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dest as *mut u8, len);
            *dest.add(len) = 0;
        }
        self.count += 1;
        Ok(())
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.count
    }
}

pub type CompletionCallback =
    unsafe extern "C" fn(partial: *const c_char, commands: *mut c_char) -> c_int;

#[repr(C)]
pub struct ConCommand {
    vtable: *const c_void,
    next: *mut ConCommand,
    is_registered: bool,
    _pad: [u8; 3],
    name: *const c_char,
    help_text: *const c_char,
    flags: i32,
    callback: extern "C" fn(*const c_void, *const CCommand),
    completion_cb: Option<CompletionCallback>,
    has_completion: bool,
    _pad2: [u8; 3],
}

impl ConCommand {
    pub const fn new(
        name: &'static str,
        help_text: &'static str,
        callback: extern "C" fn(*const c_void, *const CCommand),
        completion_cb: Option<CompletionCallback>,
    ) -> Self {
        Self {
            vtable: ptr::null(),
            next: ptr::null_mut(),
            is_registered: false,
            _pad: [0; 3],
            name: name.as_bytes().as_ptr() as *const c_char,
            help_text: help_text.as_bytes().as_ptr() as *const c_char,
            flags: 0,
            callback,
            completion_cb,
            has_completion: completion_cb.is_some(),
            _pad2: [0; 3],
        }
    }
}

#[repr(transparent)]
pub struct SyncCommand(UnsafeCell<ConCommand>);

unsafe impl Send for SyncCommand {}
unsafe impl Sync for SyncCommand {}

impl SyncCommand {
    #[inline]
    pub const fn new(cmd: ConCommand) -> Self {
        Self(UnsafeCell::new(cmd))
    }

    #[inline]
    pub const fn get(&self) -> *mut ConCommand {
        self.0.get()
    }
}

#[repr(C)]
struct ICvarVTable {
    slots: [*const c_void; 32],
}

#[repr(C)]
struct ICvar {
    vtable: *const ICvarVTable,
}

pub fn register_plugin_commands(interface_factory: CreateInterfaceFn) {
    let mut status = 0;

    let cvar_ptr = unsafe {
        interface_factory(c"VEngineCvar007".as_ptr() as *const c_char, &mut status) as *mut ICvar
    };

    ICVAR.store(cvar_ptr as usize, Ordering::Relaxed);

    if cvar_ptr.is_null() {
        elog!("VEngineCvar007 interface failed to obtain");
        clog!(err_prefix;
            "Failed to obtain VEngineCvar007 interface! \
            Commands will be unavailable"
        );
        return;
    }

    let cvar_vtable = unsafe { (*cvar_ptr).vtable };

    let mut native_vtable: *const c_void = ptr::null();
    let target_cmd_name = c"help".as_ptr() as *const c_char;

    let find_fn: extern "thiscall" fn(*mut c_void, *const c_char) -> *mut c_void =
        unsafe { std::mem::transmute(cvar_vtable.as_ref_unchecked().slots[14]) };

    let existing_cmd = find_fn(cvar_ptr as *mut c_void, target_cmd_name);

    if !existing_cmd.is_null() && (existing_cmd as usize) > 0x10000 {
        let vtable_ptr = unsafe { *(existing_cmd as *const *const c_void) };
        if !vtable_ptr.is_null() && (vtable_ptr as usize) > 0x10000 {
            native_vtable = vtable_ptr;
            log!("Borrowed native ConCommand VTable via ICvar Slot 14");
        }
    }

    if native_vtable.is_null() {
        elog!("Native ConCommand VTable not found");
        clog!(err_prefix;
            "Could not locate native ConCommand VTable in memory! \
            Commands will be unavailable"
        );
        return;
    }

    let curr_build_index = engine_build::current_build() as usize;
    for (sync_cmd, builds) in &crate::commands::PLUGIN_COMMANDS {
        if builds[curr_build_index].is_none() {
            continue;
        }
        let cmd_ptr = sync_cmd.get();
        unsafe {
            (*cmd_ptr).vtable = native_vtable;
        }

        let register_fn: extern "thiscall" fn(*mut c_void, *mut ConCommand) = unsafe {
            std::mem::transmute(cvar_vtable.as_ref_unchecked().slots[ICVAR_REGISTER_COMMAND_SLOT])
        };

        register_fn(cvar_ptr as *mut c_void, cmd_ptr);

        unsafe {
            if !(*cmd_ptr).is_registered {
                elog!("Unable to register {:?}", (*cmd_ptr).name);
                break;
            }
        }
    }
}

pub fn unregister_plugin_commands() {
    let cvar_ptr = ICVAR.load(Ordering::Relaxed) as *mut ICvar;
    if cvar_ptr.is_null() {
        return;
    }
    let cvar_vtable = unsafe { (*cvar_ptr).vtable };
    if cvar_vtable.is_null() {
        return;
    }

    let curr_build_index = engine_build::current_build() as usize;
    for (sync_cmd, builds) in &crate::commands::PLUGIN_COMMANDS {
        if builds[curr_build_index].is_none() {
            continue;
        }
        let cmd_ptr = sync_cmd.get();

        let unregister_fn: extern "thiscall" fn(*mut c_void, *mut ConCommand) = unsafe {
            std::mem::transmute(cvar_vtable.as_ref_unchecked().slots[ICVAR_UNREGISTER_COMMAND_SLOT])
        };

        unregister_fn(cvar_ptr as *mut c_void, cmd_ptr);
    }
}
