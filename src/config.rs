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

use gdal_sys::{CPLErr, CPLErrorNum, CPLGetErrorHandlerUserData};
use libc::{c_char, c_void};

use crate::errors::{CplErrType, Result};
use crate::utils::_string;
use once_cell::sync::Lazy;
use std::ffi::CString;
use std::sync::Mutex;

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
    Ok(_string(rv))
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

/// Set a GDAL library configuration option
/// with **thread local** scope
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
///
pub fn set_thread_local_config_option(key: &str, value: &str) -> Result<()> {
    let c_key = CString::new(key.as_bytes())?;
    let c_val = CString::new(value.as_bytes())?;
    unsafe {
        gdal_sys::CPLSetThreadLocalConfigOption(c_key.as_ptr(), c_val.as_ptr());
    };
    Ok(())
}

/// Get the value of a GDAL library configuration option
/// with **thread local** scope
///
/// If the config option specified by `key` is not found, the value passed in the `default` paramter is returned.
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn get_thread_local_config_option(key: &str, default: &str) -> Result<String> {
    let c_key = CString::new(key.as_bytes())?;
    let c_default = CString::new(default.as_bytes())?;
    let rv = unsafe { gdal_sys::CPLGetThreadLocalConfigOption(c_key.as_ptr(), c_default.as_ptr()) };
    Ok(_string(rv))
}

/// Clear the value of a GDAL library configuration option
/// with **thread local** scope
///
/// Refer to [GDAL `ConfigOptions`](https://trac.osgeo.org/gdal/wiki/ConfigOptions) for
/// a full list of options.
pub fn clear_thread_local_config_option(key: &str) -> Result<()> {
    let c_key = CString::new(key.as_bytes())?;
    unsafe {
        gdal_sys::CPLSetThreadLocalConfigOption(c_key.as_ptr(), ::std::ptr::null());
    };
    Ok(())
}

type ErrorCallbackType = dyn FnMut(CplErrType, i32, &str) + 'static + Send;
// We have to double-`Box` the type because we need two things:
// 1. A stable pointer for moving the data in and out of the `Mutex`. This is done by the outer `Box`.
// 2. A thin pointer to our Trait-`FnMut`. This is done by the inner (sized) `Box`. We cannot use `*mut dyn FnMut`
//    (a fat pointer) since we have to cast it from a `*mut c_void`, which is a thin pointer.
type PinnedErrorCallback = Box<Box<ErrorCallbackType>>;

/// Static variable that holds the current error callback function
static ERROR_CALLBACK: Lazy<Mutex<Option<PinnedErrorCallback>>> = Lazy::new(Default::default);

/// Set a custom error handler for GDAL.
/// Could be overwritten by setting a thread-local error handler.
///
// Note:
// Stores the callback in the static variable [`ERROR_CALLBACK`].
// Internally, it passes a pointer to the callback to GDAL as `pUserData`.
//
/// The function must be `Send` and `Sync` since it is potentially called from multiple threads.
///
pub fn set_error_handler<F>(callback: F)
where
    F: FnMut(CplErrType, i32, &str) + 'static + Send + Sync,
{
    unsafe extern "C" fn error_handler(
        error_type: CPLErr::Type,
        error_num: CPLErrorNum,
        error_msg_ptr: *const c_char,
    ) {
        let error_msg = _string(error_msg_ptr);
        let error_type: CplErrType = error_type.into();

        // reconstruct callback from user data pointer
        let callback_raw = CPLGetErrorHandlerUserData();
        let callback: &mut Box<ErrorCallbackType> = &mut *(callback_raw as *mut Box<_>);

        callback(error_type, error_num, &error_msg);
    }

    // pin memory location of callback for sending its pointer to GDAL
    let mut callback: PinnedErrorCallback = Box::new(Box::new(callback));

    let callback_ref: &mut Box<ErrorCallbackType> = callback.as_mut();

    let mut callback_lock = match ERROR_CALLBACK.lock() {
        Ok(guard) => guard,
        // poisoning could only occur on `CPLSetErrorHandler(Ex)` panicing, thus the value must be valid nevertheless
        Err(poison_error) => poison_error.into_inner(),
    };

    // changing the error callback is fenced by the callback lock
    unsafe {
        gdal_sys::CPLSetErrorHandlerEx(Some(error_handler), callback_ref as *mut _ as *mut c_void);
    };

    // store callback in static variable so we avoid a dangling pointer
    callback_lock.replace(callback);
}

/// Remove a custom error handler for GDAL.
pub fn remove_error_handler() {
    let mut callback_lock = match ERROR_CALLBACK.lock() {
        Ok(guard) => guard,
        // poisoning could only occur on `CPLSetErrorHandler(Ex)` panicing, thus the value must be valid nevertheless
        Err(poison_error) => poison_error.into_inner(),
    };

    // changing the error callback is fenced by the callback lock
    unsafe {
        gdal_sys::CPLSetErrorHandler(None);
    };

    // drop callback
    callback_lock.take();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_options() {
        // We cannot test different global config scenarios in parallel since we modify a global config state in GDAL.
        // Therefore, we test the config option behavior sequentially to avoid data races.

        test_set_get_option();

        test_set_option_with_embedded_nul();

        test_clear_option();

        test_set_get_option_thread_local();

        test_set_option_with_embedded_nul_thread_local();

        test_clear_option_thread_local();
    }

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

    fn test_set_option_with_embedded_nul() {
        assert!(set_config_option("f\0oo", "valid").is_err());
        assert!(set_config_option("foo", "in\0valid").is_err());
        assert!(set_config_option("xxxf\0oo", "in\0valid").is_err());
    }

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

    fn test_set_get_option_thread_local() {
        assert!(set_thread_local_config_option("GDAL_CACHEMAX", "128").is_ok());

        assert_eq!(
            get_thread_local_config_option("GDAL_CACHEMAX", "").unwrap_or_else(|_| "".to_string()),
            "128"
        );
        // test override for global getter
        assert_eq!(
            get_config_option("GDAL_CACHEMAX", "").unwrap_or_else(|_| "".to_string()),
            "128"
        );

        assert_eq!(
            get_thread_local_config_option("NON_EXISTANT_OPTION", "DEFAULT_VALUE")
                .unwrap_or_else(|_| "".to_string()),
            "DEFAULT_VALUE"
        );
    }

    fn test_set_option_with_embedded_nul_thread_local() {
        assert!(set_thread_local_config_option("f\0oo", "valid").is_err());
        assert!(set_thread_local_config_option("foo", "in\0valid").is_err());
        assert!(set_thread_local_config_option("xxxf\0oo", "in\0valid").is_err());
    }

    fn test_clear_option_thread_local() {
        assert!(set_thread_local_config_option("TEST_OPTION", "256").is_ok());

        assert_eq!(
            get_thread_local_config_option("TEST_OPTION", "DEFAULT")
                .unwrap_or_else(|_| "".to_string()),
            "256"
        );
        // test override for global getter
        assert_eq!(
            get_config_option("TEST_OPTION", "DEFAULT").unwrap_or_else(|_| "".to_string()),
            "256"
        );

        assert!(clear_thread_local_config_option("TEST_OPTION").is_ok());

        assert_eq!(
            get_thread_local_config_option("TEST_OPTION", "DEFAULT")
                .unwrap_or_else(|_| "".to_string()),
            "DEFAULT"
        );
        // test override for global getter
        assert_eq!(
            get_config_option("TEST_OPTION", "DEFAULT").unwrap_or_else(|_| "".to_string()),
            "DEFAULT"
        );
    }
}
