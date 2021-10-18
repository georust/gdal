use std::ffi::CString;
use std::ptr;

use gdal_sys::CSLSetNameValue;
use libc::c_char;

use crate::errors::{GdalError, Result};
use crate::utils::_string;

/// Wraps a `char **papszStrList` pointer into a struct that
/// automatically destroys the allocated memory on `drop`.
///
/// See the `CSL` GDAL functions for more details.
pub struct CslStringList {
    list_ptr: *mut *mut c_char,
}

impl CslStringList {
    pub fn new() -> Self {
        Self {
            list_ptr: ptr::null_mut(),
        }
    }

    /// Assigns `value` to `name`.
    ///
    /// Overwrites duplicate `name`s.
    pub fn set_name_value(&mut self, name: &str, value: &str) -> Result<()> {
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(GdalError::BadArgument(format!(
                "Invalid characters in name: '{}'",
                name
            )));
        }
        if value.contains(|c| c == '\n' || c == '\r') {
            return Err(GdalError::BadArgument(format!(
                "Invalid characters in value: '{}'",
                value
            )));
        }
        let psz_name = CString::new(name)?;
        let psz_value = CString::new(value)?;

        unsafe {
            self.list_ptr = CSLSetNameValue(self.list_ptr, psz_name.as_ptr(), psz_value.as_ptr());
        }

        Ok(())
    }

    /// Looks up the value corresponding to a key.
    ///
    /// See `CSLFetchNameValue` for details.
    pub fn fetch_name_value(&self, key: &str) -> Result<Option<String>> {
        let key = CString::new(key)?;
        let c_value = unsafe { gdal_sys::CSLFetchNameValue(self.as_ptr(), key.as_ptr()) };
        let value = if c_value.is_null() {
            None
        } else {
            Some(_string(c_value))
        };
        Ok(value)
    }

    pub fn as_ptr(&self) -> gdal_sys::CSLConstList {
        self.list_ptr
    }
}

impl Drop for CslStringList {
    fn drop(&mut self) {
        unsafe { gdal_sys::CSLDestroy(self.list_ptr) }
    }
}

impl Default for CslStringList {
    fn default() -> Self {
        Self::new()
    }
}
