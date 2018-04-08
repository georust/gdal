use libc::c_int;
use std::ffi::CString;
use std::ptr::null_mut;
use std::sync::{Once, ONCE_INIT};
use utils::{_string, _last_null_pointer_err};
use raster::Dataset;
use raster::types::GdalType;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{self, GDALDriverH, GDALMajorObjectH};

use errors::*;

static START: Once = ONCE_INIT;

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::GDALAllRegister();
        });
    }
}


#[allow(missing_copy_implementations)]
pub struct Driver {
    c_driver: GDALDriverH,
}


impl Driver {
    pub fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            Err(_last_null_pointer_err("GDALGetDriverByName"))?;
        };
        Ok(Driver{c_driver})
    }

    pub unsafe fn _with_c_ptr(c_driver: GDALDriverH) -> Driver {
        Driver { c_driver }
    }

    pub unsafe fn _c_ptr(&self) -> GDALDriverH {
        self.c_driver
    }

    pub fn short_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverShortName(self.c_driver) };
        _string(rv)
    }

    pub fn long_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverLongName(self.c_driver) };
        _string(rv)
    }

    pub fn create(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8>(
            filename,
            size_x,
            size_y,
            bands,
        )
    }

    pub fn create_with_band_type<T: GdalType>(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe { gdal_sys::GDALCreate(
                self.c_driver,
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                T::gdal_type(),
                null_mut()
            ) };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("GDALCreate"))?;
        };
        Ok(unsafe { Dataset::_with_c_ptr(c_dataset) })
    }
}

impl MajorObject for Driver {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}
