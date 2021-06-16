use crate::dataset::Dataset;
use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::{GdalType, RasterCreationOption};
use crate::utils::{_last_null_pointer_err, _string};
use gdal_sys::{self, GDALDriverH, GDALMajorObjectH};
use libc::{c_char, c_int};
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

type CSLConstList = *mut *mut c_char;

extern "C" {
    fn CSLSetNameValue(
        papszOptions: CSLConstList,
        pszName: *const libc::c_char,
        pszValue: *const libc::c_char,
    ) -> *mut *mut libc::c_char;
}

impl Driver {
    pub fn get(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            return Err(_last_null_pointer_err("GDALGetDriverByName"));
        };
        Ok(Driver { c_driver })
    }

    /// Creates a new Driver object by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_driver(c_driver: GDALDriverH) -> Driver {
        Driver { c_driver }
    }

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_driver(&self) -> GDALDriverH {
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
        bands: isize,
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8>(filename, size_x, size_y, bands)
    }

    pub fn create_with_band_type<T: GdalType>(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        let options = [];
        self.create_with_band_type_with_options::<T>(filename, size_x, size_y, bands, &options)
    }

    pub fn create_with_band_type_with_options<T: GdalType>(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        let mut options_c = null_mut();
        for option in options {
            let psz_name = CString::new(option.key)?;
            let psz_value = CString::new(option.value)?;
            unsafe {
                options_c = CSLSetNameValue(options_c, psz_name.as_ptr(), psz_value.as_ptr());
            }
        }

        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreate(
                self.c_driver,
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                T::gdal_type(),
                options_c as *mut *mut i8,
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALCreate"));
        };
        Ok(unsafe { Dataset::from_c_dataset(c_dataset) })
    }

    pub fn create_vector_only(&self, filename: &str) -> Result<Dataset> {
        self.create_with_band_type::<u8>(filename, 0, 0, 0)
    }
}

impl MajorObject for Driver {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}
