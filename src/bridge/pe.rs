use std::ptr;

const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D; // "MZ"
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550; // "PE\0\0"

/// Reads the PE `TimeDateStamp` from a loaded module base address.
/// The timestamp is Unix epoch seconds from when the module was linked.
pub fn module_timestamp(base: usize) -> Option<u32> {
    if base == 0 {
        return None;
    }

    unsafe {
        let dos = base as *const u8;

        let dos_sig = ptr::read_unaligned(dos as *const u16);
        if dos_sig != IMAGE_DOS_SIGNATURE {
            return None;
        }

        let pe_off = ptr::read_unaligned(dos.add(0x3C) as *const i32);
        if pe_off <= 0 {
            return None;
        }

        let nt = dos.add(pe_off as usize);

        let nt_sig = ptr::read_unaligned(nt as *const u32);
        if nt_sig != IMAGE_NT_SIGNATURE {
            return None;
        }

        Some(ptr::read_unaligned(nt.add(8) as *const u32))
    }
}
