use crate::gdal_common::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::types::GdalType;
use crate::raster::{Dataset, DatasetExt};
use crate::utils::{_last_null_pointer_err, _string};
use gdal_sys::{self, GDALDriverH, GDALMajorObjectH};
use libc::c_int;
use std::ffi::CString;
use std::ptr::null_mut;
use std::sync::Once;

use crate::errors::*;

static START: Once = Once::new();

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

pub trait DriverExt {
    
    unsafe fn c_driver(&self) -> GDALDriverH;
    
    fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            Err(_last_null_pointer_err("GDALGetDriverByName"))?;
        };
        Ok(Driver { c_driver })
    }

    unsafe fn from_c_ptr(c_driver: GDALDriverH) -> Driver {
        Driver { c_driver }
    }

    fn short_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverShortName(self.c_driver()) };
        _string(rv)
    }

    fn long_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverLongName(self.c_driver()) };
        _string(rv)
    }

    fn create(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8>(filename, size_x, size_y, bands)
    }

    fn create_with_band_type<T: GdalType>(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreate(
                self.c_driver(),
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                T::gdal_type(),
                null_mut(),
            )
        };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("GDALCreate"))?;
        };
        Ok(unsafe { Dataset::from_c_ptr(c_dataset) })
    }
}

impl DriverExt for Driver {
    unsafe fn c_driver(&self) -> GDALDriverH {
        self.c_driver
    }
    
}

impl MajorObject for Driver {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}
