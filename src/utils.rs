use libc::c_char;
use std::ffi::CStr;
use gdal_sys::cpl_error;

use errors::*;


pub fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    c_str.to_string_lossy().into_owned()
}


// TODO: inspect if this is sane...
pub fn _last_cpl_err(cpl_err_class: cpl_error::CPLErr) -> ErrorKind {
    let last_err_no = unsafe { cpl_error::CPLGetLastErrorNo() };
    let last_err_msg = _string( unsafe { cpl_error::CPLGetLastErrorMsg() } );
    unsafe { cpl_error::CPLErrorReset() };
    ErrorKind::CplError(cpl_err_class, last_err_no, last_err_msg)
}

pub fn _last_null_pointer_err(method_name: &'static str) -> ErrorKind {
    let last_err_msg = _string( unsafe { cpl_error::CPLGetLastErrorMsg() } );
    unsafe { cpl_error::CPLErrorReset() };
    ErrorKind::NullPointer(method_name, last_err_msg)
}