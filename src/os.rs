use std::os::raw::{c_char, c_void};

unsafe extern "system" {
    pub fn GetModuleHandleA(lpModuleName: *const c_char) -> *mut c_void;
    pub fn GetLocalTime(lpSystemTime: *mut Systemtime);
}

#[repr(C)]
#[derive(Default)]
pub struct Systemtime {
    pub w_year: u16,
    pub w_month: u16,
    pub w_day_of_week: u16,
    pub w_day: u16,
    pub w_hour: u16,
    pub w_minute: u16,
    pub w_second: u16,
    pub w_milliseconds: u16,
}

pub fn format_local_time(fmt: &str) -> String {
    let mut st = Systemtime::default();
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
