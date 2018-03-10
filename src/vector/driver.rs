use std::ffi::CString;
use std::ptr::null_mut;
use std::sync::{Once, ONCE_INIT};
use std::path::Path;
use vector::{Dataset};
use gdal_sys::{self, OGRSFDriverH};
use utils::{_last_null_pointer_err};

use errors::*;


static START: Once = ONCE_INIT;

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::OGRRegisterAll();
        });
    }
}

pub struct Driver {
    c_driver: OGRSFDriverH,
}

impl Driver {
    pub fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::OGRGetDriverByName(c_name.as_ptr()) };
       if c_driver.is_null() {
            Err(_last_null_pointer_err("OGRGetDriverByName").into())
        } else {
            Ok(Driver{c_driver: c_driver})
        }
    }

    pub fn create(&self, path: &Path) -> Result<Dataset> {
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_dataset = unsafe { gdal_sys::OGR_Dr_CreateDataSource(
            self.c_driver,
            c_filename.as_ptr(),
            null_mut(),
        ) };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("OGR_Dr_CreateDataSource").into())
        } else {
            Ok( unsafe { Dataset::_with_c_dataset(c_dataset) } )
        }
    }
}
