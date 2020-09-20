//! GDAL Configuration Functions
//!
//! The GDAL library can be configured at runtime using environment variables or
//! by using functions in this module. Options set by calling functions in this
//! module override options set in environment variables.
//!
//! ```
//! use gdal::config::*;
//!
//! // Increase GDAL's cache size to 1024Mb
//! set_config_option("GDAL_CACHEMAX", "1024").unwrap();
//!
//! // Get the size of GDAL's cache
//! assert_eq!(get_config_option("GDAL_CACHEMAX", "").unwrap(), "1024");
//!
//! // Set the cache size back to default
//! clear_config_option("GDAL_CACHEMAX").unwrap();
//!
//! // Check the option has been cleared
//! assert_eq!(get_config_option("GDAL_CACHEMAX", "XXX").unwrap(), "XXX");
//! ```
//!
//! Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
//! a full list of options.

use crate::errors::Result;
use crate::utils::_string;
use std::ffi::CString;

/// Set a GDAL library configuration option
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
///
pub fn set_config_option(key: &str, value: &str) -> Result<()> {
    let c_key = CString::new(key.as_bytes())?;
    let c_val = CString::new(value.as_bytes())?;
    unsafe {
        gdal_sys::CPLSetConfigOption(c_key.as_ptr(), c_val.as_ptr());
    };
    Ok(())
}

/// Get the value of a GDAL library configuration option
///
/// If the config option specified by `key` is not found, the value passed in the `default` paramter is returned.
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn get_config_option(key: &str, default: &str) -> Result<String> {
    let c_key = CString::new(key.as_bytes())?;
    let c_default = CString::new(default.as_bytes())?;
    let rv = unsafe { gdal_sys::CPLGetConfigOption(c_key.as_ptr(), c_default.as_ptr()) };
    let rstr = unsafe { _string(rv) };
    Ok(rstr)
}

/// Clear the value of a GDAL library configuration option
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn clear_config_option(key: &str) -> Result<()> {
    let c_key = CString::new(key.as_bytes())?;
    unsafe {
        gdal_sys::CPLSetConfigOption(c_key.as_ptr(), ::std::ptr::null());
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    //use std::ffi::NulError;

    #[test]
    fn test_set_get_option() {
        assert!(set_config_option("GDAL_CACHEMAX", "128").is_ok());
        assert_eq!(
            get_config_option("GDAL_CACHEMAX", "").unwrap_or_else(|_| "".to_string()),
            "128"
        );
        assert_eq!(
            get_config_option("NON_EXISTANT_OPTION", "DEFAULT_VALUE")
                .unwrap_or_else(|_| "".to_string()),
            "DEFAULT_VALUE"
        );
    }

    #[test]
    fn test_set_option_with_embedded_nul() {
        assert!(set_config_option("f\0oo", "valid").is_err());
        assert!(set_config_option("foo", "in\0valid").is_err());
        assert!(set_config_option("xxxf\0oo", "in\0valid").is_err());
    }

    #[test]
    fn test_clear_option() {
        assert!(set_config_option("TEST_OPTION", "256").is_ok());
        assert_eq!(
            get_config_option("TEST_OPTION", "DEFAULT").unwrap_or_else(|_| "".to_string()),
            "256"
        );
        assert!(clear_config_option("TEST_OPTION").is_ok());
        assert_eq!(
            get_config_option("TEST_OPTION", "DEFAULT").unwrap_or_else(|_| "".to_string()),
            "DEFAULT"
        );
    }
}
