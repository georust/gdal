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

use crate::errors::Result;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum CplErr {
    None = 0,
    Debug = 1,
    Warning = 2,
    Failure = 3,
    Fatal = 4,
}

type CallbackType = dyn FnMut(CplErr, i32, &str) + 'static;
static mut ERROR_CALLBACK: Option<Box<CallbackType>> = None;

type CallbackTypeThreadSafe = dyn FnMut(CplErr, i32, &str) + 'static + Send;

static ERROR_CALLBACK_THREAD_SAFE: Lazy<Mutex<Option<Box<CallbackTypeThreadSafe>>>> =
    Lazy::new(Default::default);

/// Set a custom error handler for GDAL.
/// Could be overwritten by setting a thread-local error handler.
///
/// *This method is not thread safe.*
///
/// Note:
/// Stores the callback in the static variable [`ERROR_CALLBACK`].
/// Internally, it passes a pointer to the callback to GDAL as `pUserData`.
///
pub fn set_error_handler<F>(callback: F)
where
    F: FnMut(CplErr, i32, &str) + 'static,
{
    unsafe extern "C" fn error_handler(
        error_type: CPLErr::Type,
        error_num: CPLErrorNum,
        error_msg_ptr: *const c_char,
    ) {
        let error_msg = _string(error_msg_ptr);
        let error_type: CplErr = std::mem::transmute(error_type);

        // reconstruct callback from user data pointer
        let callback_raw = CPLGetErrorHandlerUserData();
        let callback: &mut Box<CallbackType> = &mut *(callback_raw as *mut Box<_>);

        callback(error_type, error_num as i32, &error_msg);
    }

    unsafe {
        // this part is not thread safe
        ERROR_CALLBACK.replace(Box::new(callback));

        if let Some(callback) = &mut ERROR_CALLBACK {
            gdal_sys::CPLSetErrorHandlerEx(Some(error_handler), callback as *mut _ as *mut c_void);
        }
    };
}

/// Set a custom error handler for GDAL.
/// Could be overwritten by setting a thread-local error handler.
///
/// Note:
/// Stores the callback in the static variable [`ERROR_CALLBACK_THREAD_SAFE`].
/// Internally, it passes a pointer to the callback to GDAL as `pUserData`.
///
pub fn set_error_handler_thread_safe<F>(callback: F)
where
    F: FnMut(CplErr, i32, &str) + 'static + Send,
{
    unsafe extern "C" fn error_handler(
        error_type: CPLErr::Type,
        error_num: CPLErrorNum,
        error_msg_ptr: *const c_char,
    ) {
        let error_msg = _string(error_msg_ptr);
        let error_type: CplErr = std::mem::transmute(error_type);

        // reconstruct callback from user data pointer
        let callback_raw = CPLGetErrorHandlerUserData();
        let callback: &mut Box<CallbackTypeThreadSafe> = &mut *(callback_raw as *mut Box<_>);

        callback(error_type, error_num as i32, &error_msg);
    }

    let mut callback_lock = match ERROR_CALLBACK_THREAD_SAFE.lock() {
        Ok(guard) => guard,
        // poor man's lock poisoning handling, i.e., ignoring it
        Err(poison_error) => poison_error.into_inner(),
    };
    callback_lock.replace(Box::new(callback));

    if let Some(callback) = callback_lock.as_mut() {
        unsafe {
            gdal_sys::CPLSetErrorHandlerEx(Some(error_handler), callback as *mut _ as *mut c_void);
        };
    }
}

/// Remove a custom error handler for GDAL.
pub fn remove_error_handler() {
    unsafe {
        gdal_sys::CPLSetErrorHandler(None);
    };

    // drop callback
    unsafe {
        ERROR_CALLBACK.take();
    }
}

/// Remove a custom error handler for GDAL.
pub fn remove_error_handler_thread_safe() {
    unsafe {
        gdal_sys::CPLSetErrorHandler(None);
    };

    // drop callback

    let mut callback_lock = match ERROR_CALLBACK_THREAD_SAFE.lock() {
        Ok(guard) => guard,
        // poor man's lock poisoning handling, i.e., ignoring it
        Err(poison_error) => poison_error.into_inner(),
    };

    callback_lock.take();
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn error_handler() {
        let errors: Arc<Mutex<Vec<(CplErr, i32, String)>>> = Arc::new(Mutex::new(vec![]));

        let errors_clone = errors.clone();

        set_error_handler(move |a, b, c| {
            errors_clone.lock().unwrap().push((a, b, c.to_string()));
        });

        unsafe {
            let msg = CString::new("foo".as_bytes()).unwrap();
            gdal_sys::CPLError(CPLErr::CE_Failure, 42, msg.as_ptr());
        };

        unsafe {
            let msg = CString::new("bar".as_bytes()).unwrap();
            gdal_sys::CPLError(std::mem::transmute(CplErr::Warning), 1, msg.as_ptr());
        };

        remove_error_handler();

        let result: Vec<(CplErr, i32, String)> = errors.lock().unwrap().clone();
        assert_eq!(
            result,
            vec![
                (CplErr::Failure, 42, "foo".to_string()),
                (CplErr::Warning, 1, "bar".to_string())
            ]
        );
    }

    #[test]
    fn error_handler_thread_safe() {
        let errors: Arc<Mutex<Vec<(CplErr, i32, String)>>> = Arc::new(Mutex::new(vec![]));

        let errors_clone = errors.clone();

        set_error_handler_thread_safe(move |a, b, c| {
            errors_clone.lock().unwrap().push((a, b, c.to_string()));
        });

        unsafe {
            let msg = CString::new("foo".as_bytes()).unwrap();
            gdal_sys::CPLError(CPLErr::CE_Failure, 42, msg.as_ptr());
        };

        unsafe {
            let msg = CString::new("bar".as_bytes()).unwrap();
            gdal_sys::CPLError(std::mem::transmute(CplErr::Warning), 1, msg.as_ptr());
        };

        remove_error_handler_thread_safe();

        let result: Vec<(CplErr, i32, String)> = errors.lock().unwrap().clone();
        assert_eq!(
            result,
            vec![
                (CplErr::Failure, 42, "foo".to_string()),
                (CplErr::Warning, 1, "bar".to_string())
            ]
        );
    }

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

    #[test]
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

    #[test]
    fn test_set_option_with_embedded_nul_thread_local() {
        assert!(set_thread_local_config_option("f\0oo", "valid").is_err());
        assert!(set_thread_local_config_option("foo", "in\0valid").is_err());
        assert!(set_thread_local_config_option("xxxf\0oo", "in\0valid").is_err());
    }

    #[test]
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
