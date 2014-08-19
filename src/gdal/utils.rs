use libc::c_char;
use std::c_str::CString;


pub fn _string(raw_ptr: *const c_char) -> String {
    let c_string = unsafe { CString::new(raw_ptr, false) };
    return String::from_str(c_string.as_str().unwrap());
}
