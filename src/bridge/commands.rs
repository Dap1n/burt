use std::{
    cell::UnsafeCell,
    ffi::{CStr, c_char, c_int, c_void},
    ptr,
};

use crate::{clog, elog, log};

pub type CreateInterfaceFn =
    unsafe extern "cdecl" fn(p_name: *const c_char, p_return_code: *mut c_int) -> *mut c_void;

#[repr(C)]
pub struct CCommand {
    argc: c_int,                 // 0x00: Number of arguments
    _argv0_size: c_int,          // 0x04: Size of command name
    arg_s_buffer: [c_char; 512], // 0x08: Full unparsed command string buffer
    arg_buffer: [c_char; 512],   // 0x208: Parsed null-separated argument string buffer
    args: [*const c_char; 64],   // 0x408: Array of pointers to arguments inside arg_buffer
}

impl CCommand {
    /// Get an argument by index as a Rust `&str` slice.
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
    pub fn len(&self) -> usize {
        self.argc as usize
    }

    pub fn is_empty(&self) -> bool {
        self.argc <= 1
    }

    /// Returns raw remaining command line starting from argument index `arg_index`.
    /// Correctly handles quotes, multiple spaces, and formatting.
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

#[repr(C)]
pub struct ConCommand {
    vtable: *const c_void,    // 0x00 - Borrowed native engine VTable
    next: *mut ConCommand,    // 0x04
    is_registered: bool,      // 0x08
    _pad: [u8; 3],            // 0x09 - 0x0B
    name: *const c_char,      // 0x0C
    help_text: *const c_char, // 0x10
    flags: i32,               // 0x14
    callback: extern "C" fn(*const c_void, *const CCommand), // 0x18
    completion_cb: *const c_void, // 0x1C
    has_completion: bool,     // 0x20
    _pad2: [u8; 3],           // 0x21 - 0x23
}

impl ConCommand {
    pub const fn new(
        name: &'static str,
        help_text: &'static str,
        callback: extern "C" fn(*const c_void, *const CCommand),
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
            completion_cb: ptr::null(),
            has_completion: false,
            _pad2: [0; 3],
        }
    }
}

#[repr(transparent)]
pub struct SyncCommand(UnsafeCell<ConCommand>);

unsafe impl Send for SyncCommand {}
unsafe impl Sync for SyncCommand {}

impl SyncCommand {
    pub const fn new(cmd: ConCommand) -> Self {
        Self(UnsafeCell::new(cmd))
    }

    pub fn get(&self) -> *mut ConCommand {
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

    if cvar_ptr.is_null() {
        elog!("VEngineCvar007 interface failed to obtain");
        clog!(prefix;
            "ERROR: Failed to obtain VEngineCvar007 interface! \
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
        clog!(prefix;
            "ERROR: Could not locate native ConCommand VTable in memory! \
            Commands will be unavailable"
        );
        return;
    }

    for sync_cmd in &crate::commands::PLUGIN_COMMANDS {
        let cmd_ptr = sync_cmd.get();
        unsafe {
            (*cmd_ptr).vtable = native_vtable;
        }

        let register_fn: extern "thiscall" fn(*mut c_void, *mut ConCommand) =
            unsafe { std::mem::transmute(cvar_vtable.as_ref_unchecked().slots[9]) };

        register_fn(cvar_ptr as *mut c_void, cmd_ptr);

        unsafe {
            if !(*cmd_ptr).is_registered {
                elog!("Unable to register {:?}", (*cmd_ptr).name);
                break;
            }
        }
    }
}
