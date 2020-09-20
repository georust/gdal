use super::gdal_major_object::MajorObject;
use crate::{
    utils::{_last_null_pointer_err, _string},
    Dataset, GdalType, Metadata, _register_drivers,
};
use gdal_sys::{self, GDALDriverH, GDALMajorObjectH};
use libc::c_int;
use std::{ffi::CString, path::Path, ptr};

use crate::errors::*;

pub struct Driver {
    c_driver: GDALDriverH,
}

impl Driver {
    /// Creates a new `Driver` from a C pointer.
    /// # Safety
    /// This function operates on a C pointer
    pub unsafe fn from_c_driver(c_driver: GDALDriverH) -> Driver {
        Driver { c_driver }
    }
}

pub trait DriverCommon {
    /// Returns the `Driver` C pointer.
    /// # Safety
    /// This function operates on a C pointer
    unsafe fn c_driver(&self) -> GDALDriverH;

    fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            return Err(_last_null_pointer_err("GDALGetDriverByName").into());
        };
        Ok(Driver { c_driver })
    }

    fn short_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverShortName(self.c_driver()) };
        unsafe { _string(rv) }
    }

    fn long_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverLongName(self.c_driver()) };
        unsafe { _string(rv) }
    }

    fn create(
        &self,
        filename: &Path,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8>(filename, size_x, size_y, bands)
    }

    fn create_vector_only(&self, filename: &Path) -> Result<Dataset> {
        self.create_with_band_type::<u8>(filename, 0, 0, 0)
    }

    fn create_with_band_type<T: GdalType>(
        &self,
        filename: &Path,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        let filename = filename.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreate(
                self.c_driver(),
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                T::gdal_type(),
                ptr::null_mut(),
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALCreate").into());
        };
        Ok(unsafe { Dataset::from_c_dataset(c_dataset) })
    }
}

impl DriverCommon for Driver {
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
