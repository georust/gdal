use std::ffi::CString;
use std::ptr::null;
use std::sync::{Once, ONCE_INIT};
use std::path::Path;
use libc::{c_void};
use vector::{Dataset};
use gdal_sys::ogr;


static START: Once = ONCE_INIT;
static mut registered_drivers: bool = false;


pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            ogr::OGRRegisterAll();
            registered_drivers = true;
        });
    }
}

pub struct Driver {
    c_driver: *const c_void,
}

impl Driver {
    pub fn get(name: &str) -> Option<Driver> {
        _register_drivers();
        let c_name = CString::new(name.as_bytes()).unwrap();
        let c_driver = unsafe { ogr::OGRGetDriverByName(c_name.as_ptr()) };
        return match c_driver.is_null() {
            true  => None,
            false => Some(Driver{c_driver: c_driver}),
        };
    }

    pub fn create(&self, path: &Path) -> Option<Dataset> {
        let filename = path.to_str().unwrap();
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { ogr::OGR_Dr_CreateDataSource(
            self.c_driver,
            c_filename.as_ptr(),
            null(),
        ) };
        return match c_dataset.is_null() {
            true  => None,
            false => unsafe { Some(Dataset::_with_c_dataset(c_dataset)) },
        };
    }
}
