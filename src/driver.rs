use std::ffi::CString;
use std::path::Path;
use std::sync::Once;

use gdal_sys::{self, CPLErr, GDALDriverH, GDALMajorObjectH};
use libc::c_int;

use crate::cpl::CslStringList;
use crate::dataset::Dataset;
use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::{GdalType, RasterCreationOption};
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string, _string};

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
        let mut options_c = CslStringList::new();
        for option in options {
            options_c.set_name_value(option.key, option.value)?;
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
                options_c.as_ptr(),
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

    /// Delete named dataset.
    ///
    /// It is unwise to have open dataset handles on this dataset when it is deleted.
    ///
    /// Calls `GDALDeleteDataset()`
    ///
    pub fn delete(&self, filename: &Path) -> Result<()> {
        let c_filename = _path_to_c_string(filename)?;

        let rv = unsafe { gdal_sys::GDALDeleteDataset(self.c_driver, c_filename.as_ptr()) };

        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        Ok(())
    }

    /// Rename a dataset.
    ///
    /// It is unwise to have open dataset handles on this dataset when it is being renamed.
    ///
    /// Calls `GDALRenameDataset()`
    ///
    pub fn rename(&self, new_filename: &Path, old_filename: &Path) -> Result<()> {
        let c_old_filename = _path_to_c_string(old_filename)?;
        let c_new_filename = _path_to_c_string(new_filename)?;

        let rv = unsafe {
            gdal_sys::GDALRenameDataset(
                self.c_driver,
                c_new_filename.as_ptr(),
                c_old_filename.as_ptr(),
            )
        };

        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        Ok(())
    }
}

impl MajorObject for Driver {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}
