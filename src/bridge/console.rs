use std::{
    ffi::CString,
    mem,
    os::raw::{c_char, c_void},
};

use crate::structs::Color;

pub type ConColorMsgFn = unsafe extern "C" fn(color: *const Color, fmt: *const c_char, ...);

pub fn get_con_color_msg() -> Option<ConColorMsgFn> {
    unsafe extern "system" {
        fn GetModuleHandleA(lpModuleName: *const c_char) -> *mut c_void;
        fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    }

    let module_name = CString::new("tier0.dll").ok()?;
    let func_name = CString::new("?ConColorMsg@@YAXABVColor@@PBDZZ").ok()?;

    unsafe {
        let module = GetModuleHandleA(module_name.as_ptr());
        if !module.is_null() {
            let func = GetProcAddress(module, func_name.as_ptr());
            if !func.is_null() {
                return Some(mem::transmute::<*mut c_void, ConColorMsgFn>(func));
            }
        }
    }

    None
}

pub fn con_color_msg(func: ConColorMsgFn, color: &Color, msg: &str) {
    let c_str = match CString::new(msg) {
        Ok(s) => s,
        Err(_) => CString::new(msg.replace('\0', "\\0")).unwrap(),
    };

    unsafe {
        func(color, c"%s".as_ptr() as *const c_char, c_str.as_ptr());
    }
}
