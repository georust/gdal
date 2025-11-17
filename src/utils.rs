use std::ffi::{c_char, CStr, CString};
use std::path::{Path, PathBuf};

use gdal_sys::CPLErr;

use crate::errors::*;

/// Makes a copy of a C string, returns `None` for a null pointer.
pub fn _string(raw_ptr: *const c_char) -> Option<String> {
    if raw_ptr.is_null() {
        None
    } else {
        let c_str = unsafe { CStr::from_ptr(raw_ptr) };
        Some(c_str.to_string_lossy().into_owned())
    }
}

/// Converts an array of C strings to Rust `String`s, skipping any null pointers.
pub fn _string_array(raw_ptr: *mut *mut c_char) -> Vec<String> {
    _convert_raw_ptr_array(raw_ptr, _string)
}

/// Makes a `PathBuf` from a C string, returns `None` for a null pointer.
pub fn _pathbuf(raw_ptr: *const c_char) -> Option<PathBuf> {
    if raw_ptr.is_null() {
        None
    } else {
        let c_str = unsafe { CStr::from_ptr(raw_ptr) };
        Some(c_str.to_string_lossy().into_owned().into())
    }
}

/// Converts an array of C strings to Rust `PathBuf`s, skipping any null pointers.
pub fn _pathbuf_array(raw_ptr: *mut *mut c_char) -> Vec<PathBuf> {
    _convert_raw_ptr_array(raw_ptr, _pathbuf)
}

/// Converts an array of C strings, skipping any null pointers.
fn _convert_raw_ptr_array<F, R>(raw_ptr: *mut *mut c_char, convert: F) -> Vec<R>
where
    F: Fn(*const c_char) -> Option<R>,
{
    let mut ret_val = Vec::new();
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
            if let Some(value) = convert(next) {
                ret_val.push(value);
            }
            i += 1;
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
        msg: last_err_msg.unwrap_or_default(),
    }
}

pub fn _last_null_pointer_err(method_name: &'static str) -> GdalError {
    let last_err_msg = _string(unsafe { gdal_sys::CPLGetLastErrorMsg() });
    unsafe { gdal_sys::CPLErrorReset() };
    GdalError::NullPointer {
        method_name,
        msg: last_err_msg.unwrap_or_default(),
    }
}

pub fn _path_to_c_string(path: &Path) -> Result<CString> {
    let path_str = path.to_string_lossy();
    CString::new(path_str.as_ref()).map_err(Into::into)
}
