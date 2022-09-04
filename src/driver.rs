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

/// Raster and Vector Driver API
///
/// One of GDAL's major strengths is the vast number of data formats it's able to work with.
/// The GDAL Manual has a full list of available [raster](https://gdal.org/drivers/raster/index.html)
/// and [vector](https://gdal.org/drivers/vector/index.html) drivers.
///
/// However, due to conditional compilation, not every driver listed will necessarily be available at runtime.
/// Therefore, one of the primary uses of the the [`Driver`] is to inspect and load the available drivers.
/// (You can use `gdalinfo --formats` to peruse this list from a CLI installation of GDAL)
///
/// Each driver has its own set of options, capabilities, and limitations.
/// Furthermore, operations on one driver (e.g. copying a datasets) may or may not be available in another.
/// So when working with a new dataset it is important to refer to the driver's documentation for its capabilities.
///
/// See [`Driver`] for more details.
///
/// #### Example
///
/// ```rust
/// use gdal::Driver;
/// # fn main() -> gdal::errors::Result<()> {
/// let cog_driver = Driver::get_by_name("COG")?;
/// println!("{}", cog_driver.long_name());
/// # Ok(())
/// # }
/// ```
///
/// Output:
///
/// ```text
/// Cloud optimized GeoTIFF generator
/// ```
#[allow(missing_copy_implementations)]
pub struct Driver {
    c_driver: GDALDriverH,
}

impl Driver {
    /// Returns the driver with the given short name.
    pub fn get_by_name(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            return Err(_last_null_pointer_err("GDALGetDriverByName"));
        };
        Ok(Driver { c_driver })
    }

    /// Returns the driver with the given index, which must be less than the value returned by
    /// `Driver::count()`.
    pub fn get(index: usize) -> Result<Driver> {
        _register_drivers();
        let c_driver = unsafe { gdal_sys::GDALGetDriver(index.try_into().unwrap()) };
        if c_driver.is_null() {
            return Err(_last_null_pointer_err("GDALGetDriver"));
        }
        Ok(Driver { c_driver })
    }

    /// Returns the number of registered drivers.
    pub fn count() -> usize {
        _register_drivers();
        let count = unsafe { gdal_sys::GDALGetDriverCount() };
        count.try_into().unwrap()
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

    pub fn create<P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8, _>(filename, size_x, size_y, bands)
    }

    pub fn create_with_band_type<T: GdalType, P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        let options = [];
        self.create_with_band_type_with_options::<T, _>(filename, size_x, size_y, bands, &options)
    }

    pub fn create_with_band_type_with_options<T: GdalType, P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: isize,
        size_y: isize,
        bands: isize,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        Self::_create_with_band_type_with_options::<T>(
            self,
            filename.as_ref(),
            size_x,
            size_y,
            bands,
            options,
        )
    }

    fn _create_with_band_type_with_options<T: GdalType>(
        &self,
        filename: &Path,
        size_x: isize,
        size_y: isize,
        bands: isize,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        let mut options_c = CslStringList::new();
        for option in options {
            options_c.set_name_value(option.key, option.value)?;
        }

        let c_filename = _path_to_c_string(filename)?;
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

    pub fn create_vector_only<P: AsRef<Path>>(&self, filename: P) -> Result<Dataset> {
        self.create_with_band_type::<u8, _>(filename, 0, 0, 0)
    }

    /// Delete named dataset.
    ///
    /// It is unwise to have open dataset handles on this dataset when it is deleted.
    ///
    /// Calls `GDALDeleteDataset()`
    ///
    pub fn delete<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        Self::_delete(self, filename.as_ref())
    }

    fn _delete(&self, filename: &Path) -> Result<()> {
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
    pub fn rename<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        new_filename: P1,
        old_filename: P2,
    ) -> Result<()> {
        Self::_rename(self, new_filename.as_ref(), old_filename.as_ref())
    }

    fn _rename(&self, new_filename: &Path, old_filename: &Path) -> Result<()> {
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
