use libc::c_char;
use std::ffi::CStr;
use std::str;


pub fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    return str::from_utf8(c_str.to_bytes()).unwrap().to_string();
}
