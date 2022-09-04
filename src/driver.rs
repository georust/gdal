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

/// # Raster and Vector Driver API
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
#[allow(missing_copy_implementations)]
pub struct Driver {
    c_driver: GDALDriverH,
}

impl Driver {
    /// Returns the driver with the given short name or [`Err`] if not found.
    ///
    /// See also: [`count`](Self::count), [`get`](Self::get)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Driver;
    /// # fn main() -> gdal::errors::Result<()> {
    /// let cog_driver = Driver::get_by_name("COG")?;
    /// println!("{}", cog_driver.long_name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// Cloud optimized GeoTIFF generator
    /// ```
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
    ///
    /// See also: [`count`](Self::count)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Driver;
    /// # fn main() -> gdal::errors::Result<()> {
    /// assert!(Driver::count() > 0);
    /// let d = Driver::get(0)?;
    /// println!("'{}' is '{}'", d.short_name(), d.long_name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// 'VRT' is 'Virtual Raster'
    /// ```
    pub fn get(index: usize) -> Result<Driver> {
        _register_drivers();
        let c_driver = unsafe { gdal_sys::GDALGetDriver(index.try_into().unwrap()) };
        if c_driver.is_null() {
            return Err(_last_null_pointer_err("GDALGetDriver"));
        }
        Ok(Driver { c_driver })
    }

    /// Returns the number of registered drivers.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Driver;
    /// println!("{} drivers are registered", Driver::count());
    /// ```
    /// ```text
    /// 203 drivers are registered
    /// ```
    pub fn count() -> usize {
        _register_drivers();
        let count = unsafe { gdal_sys::GDALGetDriverCount() };
        count.try_into().unwrap()
    }

    /// Return the short name of a driver.
    ///
    /// For the GeoTIFF driver, this is “GTiff”
    ///
    /// See also: [`long_name`](Self::long_name).
    pub fn short_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverShortName(self.c_driver) };
        _string(rv)
    }

    /// Return the short name of a driver.
    ///
    /// For the GeoTIFF driver, this is “GeoTIFF”
    ///
    /// See also: [`short_name`](Self::short_name`).
    pub fn long_name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetDriverLongName(self.c_driver) };
        _string(rv)
    }

    /// Create a new dataset of size (`size_x`, `size_y`) and `bands` band count,
    /// and [`u8`] as the cell data type.
    ///
    /// To specify an alternative data type (e.g. [`f32`]), use [`create_with_band_type`](Self::create_with_band_type).
    ///
    /// See also: [`create_with_band_type_with_options`](Self::create_with_band_type_with_options).
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Driver;
    /// use gdal::raster::GdalType;
    /// let d = Driver::get_by_name("MEM")?;
    /// let ds = d.create("in-memory", 64, 64, 3)?;
    /// assert_eq!(ds.raster_count(), 3);
    /// assert_eq!(ds.raster_size(), (64, 64));
    /// assert_eq!(ds.rasterband(1)?.band_type(), u8::gdal_type());
    /// # Ok(())
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Result<Dataset> {
        self.create_with_band_type::<u8, _>(filename, size_x, size_y, bands)
    }

    /// Create a new dataset of size (`size_x`, `size_y`) and `bands` band count,
    /// with cell data type specified by `T`.
    ///
    /// See also: [`create`](Self::create), [`create_with_band_type_with_options`](Self::create_with_band_type_with_options).
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Driver;
    /// use gdal::raster::GdalType;
    /// let d = Driver::get_by_name("MEM")?;
    /// let ds = d.create_with_band_type::<f64, _>("in-memory", 64, 64, 3)?;
    /// assert_eq!(ds.raster_count(), 3);
    /// assert_eq!(ds.raster_size(), (64, 64));
    /// assert_eq!(ds.rasterband(1)?.band_type(), f64::gdal_type());
    /// # Ok(())
    /// # }
    /// ```
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

    /// Create a new dataset of size (`size_x`, `size_y`) and `bands` band count,
    /// with cell data type specified by `T` and extended options specified via `options`.
    /// [Per GDAL](https://gdal.org/api/gdaldriver_cpp.html#_CPPv4N10GDALDriver6CreateEPKciii12GDALDataType12CSLConstList),
    /// the set of legal options for `options` is driver specific, and there is no way to query in advance to establish legal values.a
    ///
    /// See also: [`RasterCreationOption`], [`create`](Self::create), [`create_with_band_type`](Self::create_with_band_type).
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Driver;
    /// use gdal::raster::RasterCreationOption;
    /// use gdal::raster::GdalType;
    /// use gdal::spatial_ref::SpatialRef;
    /// let d = Driver::get_by_name("BMP")?;
    /// let options = [
    ///     RasterCreationOption {
    ///         key: "WORLDFILE",
    ///         value: "YES"
    ///     }
    /// ];
    /// let mut ds = d.create_with_band_type_with_options::<u8, _>("/tmp/foo.bmp", 64, 64, 1, &options)?;
    /// ds.set_spatial_ref(&SpatialRef::from_epsg(4326)?)?;
    /// assert_eq!(ds.raster_count(), 1);
    /// assert_eq!(ds.raster_size(), (64, 64));
    /// assert_eq!(ds.rasterband(1)?.band_type(), u8::gdal_type());
    /// assert_eq!(ds.spatial_ref()?.auth_code()?, 4326);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Convenience for creating a vector-only dataset from a compatible driver.
    /// [Details](https://gdal.org/api/gdaldriver_cpp.html#_CPPv4N10GDALDriver6CreateEPKciii12GDALDataType12CSLConstList)
    pub fn create_vector_only<P: AsRef<Path>>(&self, filename: P) -> Result<Dataset> {
        self.create_with_band_type::<u8, _>(filename, 0, 0, 0)
    }

    /// Delete named dataset.
    ///
    /// It is unwise to have open dataset handles on this dataset when it is deleted.
    ///
    /// Calls [`GDALDeleteDataset()`](https://gdal.org/api/raster_c_api.html#_CPPv417GDALDeleteDataset11GDALDriverHPKc)
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
    /// Calls [`GDALRenameDataset()`](https://gdal.org/api/raster_c_api.html#_CPPv417GDALRenameDataset11GDALDriverHPKcPKc)
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
}

impl MajorObject for Driver {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}
