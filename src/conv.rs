//! GDAL CPL (Common Portability Library) Convenience Functions
use std::ffi::CString;
use utils::_string;
use gdal_sys::cpl_conv::{CPLSetConfigOption, CPLGetConfigOption};

pub fn set_config_option(key: &str, value: &str) {
    let c_key = CString::new(key.as_bytes()).unwrap();
    let c_val = CString::new(value.as_bytes()).unwrap();
    unsafe { CPLSetConfigOption(c_key.as_ptr(), c_val.as_ptr()) };
}

pub fn get_config_option(key: &str, default: &str) -> String {
    let c_key = CString::new(key.as_bytes()).unwrap();
    let c_default = CString::new(default.as_bytes()).unwrap();
    let rv = unsafe { CPLGetConfigOption(c_key.as_ptr(), c_default.as_ptr()) };
    return _string(rv);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_option() {
        set_config_option("GDAL_CACHEMAX","128");
        assert_eq!(get_config_option("GDAL_CACHEMAX", ""), "128");
        assert_eq!(get_config_option("NON_EXISTANT_OPTION", "DEFAULT_VALUE"), "DEFAULT_VALUE");
    }
}
