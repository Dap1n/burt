use std::os::raw::{c_char, c_void};

pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;

pub const MEM_COMMIT: u32 = 0x1000;
pub const PAGE_GUARD: u32 = 0x100;
pub const PAGE_NOACCESS: u32 = 0x01;

#[repr(C)]
pub struct MEMORY_BASIC_INFORMATION {
    pub base_address: *mut c_void,
    pub allocation_base: *mut c_void,
    pub allocation_protect: u32,
    pub region_size: usize,
    pub state: u32,
    pub protect: u32,
    pub type_: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct SYSTEMTIME {
    pub w_year: u16,
    pub w_month: u16,
    pub w_day_of_week: u16,
    pub w_day: u16,
    pub w_hour: u16,
    pub w_minute: u16,
    pub w_second: u16,
    pub w_milliseconds: u16,
}

unsafe extern "system" {
    pub fn GetModuleHandleA(lpModuleName: *const c_char) -> *mut c_void;

    pub fn VirtualProtect(
        lpAddress: *const c_void,
        dwSize: usize,
        flNewProtect: u32,
        lpflOldProtect: *mut u32,
    ) -> i32;
    pub fn VirtualQuery(
        lpAddress: *const c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: usize,
    ) -> usize;

    pub fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
}

pub fn format_local_time(fmt: &str) -> String {
    let mut st = SYSTEMTIME::default();
    unsafe { GetLocalTime(&mut st) };

    let mut out = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('Y') => out.push_str(&format!("{:04}", st.w_year)),
            Some('m') => out.push_str(&format!("{:02}", st.w_month)),
            Some('d') => out.push_str(&format!("{:02}", st.w_day)),
            Some('H') => out.push_str(&format!("{:02}", st.w_hour)),
            Some('M') => out.push_str(&format!("{:02}", st.w_minute)),
            Some('S') => out.push_str(&format!("{:02}", st.w_second)),
            Some('6') => {
                if chars.peek() == Some(&'f') {
                    chars.next();
                }
                out.push_str(&format!("{:03}000", st.w_milliseconds));
            }
            Some('f') => {
                out.push_str(&format!("{:03}000000", st.w_milliseconds));
            }
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }

    out
}
