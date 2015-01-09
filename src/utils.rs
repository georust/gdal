use libc::c_char;
use std::ffi::c_str_to_bytes;
use std::str;


pub fn _string(raw_ptr: *const c_char) -> String {
    let bytes = unsafe { c_str_to_bytes(&raw_ptr) };
    return str::from_utf8(bytes).unwrap().to_string();
}
