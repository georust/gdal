//! GDAL Configuration Functions
//!
//! The GDAL library can be configured at runtime using environment variables or
//! by using functions in this module. Options set by calling functions in this
//! module override options set in environment variables.
//!
//! Refer to [GDAL ConfigOptions](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
//! a full list of options.

use std::ffi::CString;
use utils::_string;
use gdal_sys::cpl_conv::{CPLSetConfigOption, CPLGetConfigOption};


/// Set a GDAL library configuration option
///
/// Refer to [GDAL ConfigOptions](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn set_config_option(key: &str, value: &str) {
    let c_key = CString::new(key.as_bytes()).unwrap();
    let c_val = CString::new(value.as_bytes()).unwrap();
    unsafe { CPLSetConfigOption(c_key.as_ptr(), c_val.as_ptr()); };
}

/// Get the value of a GDAL library configuration option
///
/// Refer to [GDAL ConfigOptions](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn get_config_option(key: &str, default: &str) -> String {
    let c_key = CString::new(key.as_bytes()).unwrap();
    let c_default = CString::new(default.as_bytes()).unwrap();
    let rv = unsafe { CPLGetConfigOption(c_key.as_ptr(), c_default.as_ptr()) };
    return _string(rv);
}

/// Clear the value of a GDAL library configuration option
///
/// Refer to [GDAL ConfigOptions](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn clear_config_option(key: &str) {
    let c_key = CString::new(key.as_bytes()).unwrap();
    unsafe { CPLSetConfigOption(c_key.as_ptr(), ::std::ptr::null()); };
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

    #[test]
    fn test_clear_option() {
        set_config_option("TEST_OPTION","256");
        assert_eq!(get_config_option("TEST_OPTION", "DEFAULT"), "256");
        clear_config_option("TEST_OPTION");
        assert_eq!(get_config_option("TEST_OPTION", "DEFAULT"), "DEFAULT");
    }
}
