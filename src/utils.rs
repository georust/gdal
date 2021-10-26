use gdal_sys::{self, CPLErr};
use libc::c_char;
use std::ffi::{CStr, CString};
use std::path::Path;

use crate::errors::*;

pub fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    c_str.to_string_lossy().into_owned()
}

pub fn _string_array(raw_ptr: *mut *mut c_char) -> Vec<String> {
    let mut ret_val: Vec<String> = vec![];
    let mut i = 0;
    unsafe {
        loop {
            let ptr = raw_ptr.add(i);
            if ptr.is_null() {
                break;
            }
            let next = ptr.read();
            if next.is_null() {
                break;
            }
            let value = _string(next);
            i += 1;
            ret_val.push(value);
        }
    }
    ret_val
}

// TODO: inspect if this is sane...
pub fn _last_cpl_err(cpl_err_class: CPLErr::Type) -> GdalError {
    let last_err_no = unsafe { gdal_sys::CPLGetLastErrorNo() };
    let last_err_msg = _string(unsafe { gdal_sys::CPLGetLastErrorMsg() });
    unsafe { gdal_sys::CPLErrorReset() };
    GdalError::CplError {
        class: cpl_err_class,
        number: last_err_no,
        msg: last_err_msg,
    }
}

pub fn _last_null_pointer_err(method_name: &'static str) -> GdalError {
    let last_err_msg = _string(unsafe { gdal_sys::CPLGetLastErrorMsg() });
    unsafe { gdal_sys::CPLErrorReset() };
    GdalError::NullPointer {
        method_name,
        msg: last_err_msg,
    }
}

pub fn _path_to_c_string<P: AsRef<Path>>(path: P) -> Result<CString> {
    let path_ref: &Path = path.as_ref();
    let path_str = path_ref.to_string_lossy();
    CString::new(path_str.as_ref()).map_err(Into::into)
}
