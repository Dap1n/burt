use std::{
    borrow::Cow,
    ffi::{CString, c_char, c_int, c_void},
    mem,
};

use crate::bridge::commands::CreateInterfaceFn;

#[inline]
const unsafe fn call_vfunc<F: Copy>(this_ptr: *mut c_void, index: usize) -> F {
    unsafe {
        let vtable = *(this_ptr as *const *const *const c_void);
        let vfunc_ptr = *vtable.add(index);
        mem::transmute_copy(&vfunc_ptr)
    }
}

#[repr(C)]
pub struct IVEngineServer {
    _opaque: [u8; 0],
}

impl IVEngineServer {
    const VTABLE_SERVER_COMMAND: usize = 38;
    const VTABLE_SERVER_EXECUTE: usize = 37;

    /// Queues a console command string into the server command buffer.
    pub unsafe fn queue_command(&mut self, cmd: *const c_char) {
        unsafe {
            type FnServerCommand =
                unsafe extern "thiscall" fn(this: *mut IVEngineServer, cmd: *const c_char);

            let func: FnServerCommand =
                call_vfunc(self as *mut _ as *mut c_void, Self::VTABLE_SERVER_COMMAND);
            func(self, cmd);
        }
    }

    /// Flushes and executes the queued server command buffer immediately.
    pub unsafe fn flush_commands(&mut self) {
        unsafe {
            type FnServerExecute = unsafe extern "thiscall" fn(this: *mut IVEngineServer);

            let func: FnServerExecute =
                call_vfunc(self as *mut _ as *mut c_void, Self::VTABLE_SERVER_EXECUTE);
            func(self);
        }
    }
}

#[derive(Debug)]
pub struct EngineServer {
    ptr: *mut IVEngineServer,
}

unsafe impl Send for EngineServer {}
unsafe impl Sync for EngineServer {}

impl EngineServer {
    /// Wraps a raw `IVEngineServer` pointer.
    pub unsafe fn from_raw(raw: *mut c_void) -> Option<Self> {
        (!raw.is_null()).then_some(Self {
            ptr: raw as *mut IVEngineServer,
        })
    }

    /// Obtains `IVEngineServer` directly from an engine `CreateInterface` export.
    pub unsafe fn from_factory(create_interface: CreateInterfaceFn) -> Option<Self> {
        let mut status_code: c_int = 0;

        unsafe {
            let ptr = create_interface(c"VEngineServer022".as_ptr(), &mut status_code);
            if !ptr.is_null() {
                Self::from_raw(ptr)
            } else {
                None
            }
        }
    }

    /// Executes a console command string.
    /// Appends a trailing `\n` if omitted, which is required by `IVEngineServer::ServerCommand`.
    pub fn execute(&self, command: &str, immediate: bool) {
        let formatted = if command.ends_with('\n') {
            Cow::Borrowed(command)
        } else {
            Cow::Owned(format!("{command}\n"))
        };

        let Ok(c_cmd) = CString::new(formatted.as_bytes()) else {
            return;
        };

        unsafe {
            let engine = &mut *self.ptr;
            engine.queue_command(c_cmd.as_ptr());
            if immediate {
                engine.flush_commands();
            }
        }
    }
}
