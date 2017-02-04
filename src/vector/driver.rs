use std::ffi::CString;
use std::ptr::null;
use std::sync::{Once, ONCE_INIT};
use std::path::Path;
use libc::{c_void};
use vector::{Dataset};
use gdal_sys::ogr;
use utils::{_last_null_pointer_err};

use errors::*;


static START: Once = ONCE_INIT;

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            ogr::OGRRegisterAll();
        });
    }
}

pub struct Driver {
    c_driver: *const c_void,
}

impl Driver {
    pub fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { ogr::OGRGetDriverByName(c_name.as_ptr()) };
       if c_driver.is_null() {
            Err(_last_null_pointer_err("OGRGetDriverByName").into())
        } else {
            Ok(Driver{c_driver: c_driver})
        }
    }

    pub fn create(&self, path: &Path) -> Result<Dataset> {
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_dataset = unsafe { ogr::OGR_Dr_CreateDataSource(
            self.c_driver,
            c_filename.as_ptr(),
            null(),
        ) };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("OGR_Dr_CreateDataSource").into())
        } else {
            Ok( unsafe { Dataset::_with_c_dataset(c_dataset) } )
        }
    }
}
