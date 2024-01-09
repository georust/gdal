use std::ffi::CString;
use std::path::Path;
use std::sync::Once;

use gdal_sys::{self, CPLErr, GDALDriverH, GDALMajorObjectH};

use crate::cpl::CslStringList;
use crate::dataset::Dataset;
use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::{GdalDataType, GdalType, RasterCreationOption};
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string, _string};

use crate::errors::*;

static START: Once = Once::new();

pub fn _register_drivers() {
    START.call_once(DriverManager::register_all);
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
    #[deprecated(note = "Please use `DriverManager::get_driver_by_name()` instead")]
    pub fn get_by_name(name: &str) -> Result<Driver> {
        DriverManager::get_driver_by_name(name)
    }

    /// Returns the driver with the given index, which must be less than the value returned by
    /// `Driver::count()`.
    #[deprecated(note = "Please use `DriverManager::get_driver()` instead")]
    pub fn get(index: usize) -> Result<Driver> {
        DriverManager::get_driver(index)
    }

    /// Returns the number of registered drivers.
    #[deprecated(note = "Please use `DriverManager::count()` instead")]
    pub fn count() -> usize {
        DriverManager::count()
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
    /// use gdal::DriverManager;
    /// use gdal::raster::GdalDataType;
    /// let d = DriverManager::get_driver_by_name("MEM")?;
    /// let ds = d.create("in-memory", 64, 64, 3)?;
    /// assert_eq!(ds.raster_count(), 3);
    /// assert_eq!(ds.raster_size(), (64, 64));
    /// assert_eq!(ds.rasterband(1)?.band_type(), GdalDataType::UInt8);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: usize,
        size_y: usize,
        bands: usize,
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
    /// use gdal::DriverManager;
    /// use gdal::raster::GdalDataType;
    /// let d = DriverManager::get_driver_by_name("MEM")?;
    /// let ds = d.create_with_band_type::<f64, _>("in-memory", 64, 64, 3)?;
    /// assert_eq!(ds.raster_count(), 3);
    /// assert_eq!(ds.raster_size(), (64, 64));
    /// assert_eq!(ds.rasterband(1)?.band_type(), GdalDataType::Float64);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_with_band_type<T: GdalType, P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: usize,
        size_y: usize,
        bands: usize,
    ) -> Result<Dataset> {
        let options = [];
        self.create_with_band_type_with_options::<T, _>(filename, size_x, size_y, bands, &options)
    }

    /// Create a new dataset of size (`size_x`, `size_y`) and `bands` band count,
    /// with cell data type specified by `T` and extended options specified via `options`.
    /// [Per GDAL](https://gdal.org/api/gdaldriver_cpp.html#_CPPv4N10GDALDriver6CreateEPKciii12GDALDataType12CSLConstList),
    /// the set of legal options for `options` is driver specific, and there is no way to query in advance to establish the valid ones.
    ///
    /// See also: [`RasterCreationOption`], [`create`](Self::create), [`create_with_band_type`](Self::create_with_band_type).
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::DriverManager;
    /// use gdal::raster::RasterCreationOption;
    /// use gdal::raster::GdalDataType;
    /// use gdal::spatial_ref::SpatialRef;
    /// let d = DriverManager::get_driver_by_name("BMP")?;
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
    /// assert_eq!(ds.rasterband(1)?.band_type(), GdalDataType::UInt8);
    /// assert_eq!(ds.spatial_ref()?.auth_code()?, 4326);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_with_band_type_with_options<T: GdalType, P: AsRef<Path>>(
        &self,
        filename: P,
        size_x: usize,
        size_y: usize,
        bands: usize,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        Self::_create_with_band_type_with_options(
            self,
            filename.as_ref(),
            size_x,
            size_y,
            bands,
            T::datatype(),
            options,
        )
    }

    fn _create_with_band_type_with_options(
        &self,
        filename: &Path,
        size_x: usize,
        size_y: usize,
        bands: usize,
        data_type: GdalDataType,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        let mut options_c = CslStringList::new();
        for option in options {
            options_c.set_name_value(option.key, option.value)?;
        }

        let size_x = libc::c_int::try_from(size_x)?;
        let size_y = libc::c_int::try_from(size_y)?;
        let bands = libc::c_int::try_from(bands)?;

        let c_filename = _path_to_c_string(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreate(
                self.c_driver,
                c_filename.as_ptr(),
                size_x,
                size_y,
                bands,
                data_type as u32,
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
        let options = [];
        self._create_with_band_type_with_options(
            filename.as_ref(),
            0,
            0,
            0,
            GdalDataType::Unknown,
            &options,
        )
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
    fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_driver
    }
}

impl Metadata for Driver {}

/// A wrapper around `GDALDriverManager`.
/// This struct helps listing and registering [`Driver`]s.
pub struct DriverManager;

impl DriverManager {
    /// Returns the number of registered drivers.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::DriverManager;
    /// println!("{} drivers are registered", DriverManager::count());
    /// ```
    /// ```text
    /// 203 drivers are registered
    /// ```
    pub fn count() -> usize {
        _register_drivers();
        let count = unsafe { gdal_sys::GDALGetDriverCount() };
        count
            .try_into()
            .expect("The returned count should be zero or positive")
    }

    /// Returns the driver with the given index, which must be less than the value returned by
    /// `DriverManager::count()`.
    ///
    /// See also: [`count`](Self::count)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::DriverManager;
    /// # fn main() -> gdal::errors::Result<()> {
    /// assert!(DriverManager::count() > 0);
    /// let d = DriverManager::get_driver(0)?;
    /// println!("'{}' is '{}'", d.short_name(), d.long_name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// 'VRT' is 'Virtual Raster'
    /// ```
    pub fn get_driver(index: usize) -> Result<Driver> {
        _register_drivers();
        let c_driver = unsafe { gdal_sys::GDALGetDriver(index.try_into().unwrap()) };
        if c_driver.is_null() {
            // `GDALGetDriver` just returns `null` and sets no error message
            return Err(GdalError::NullPointer {
                method_name: "GDALGetDriver",
                msg: "Unable to find driver".to_string(),
            });
        }
        Ok(Driver { c_driver })
    }

    /// Returns the driver with the given short name or [`Err`] if not found.
    ///
    /// See also: [`count`](Self::count), [`get`](Self::get_driver_by_name)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::DriverManager;
    /// # fn main() -> gdal::errors::Result<()> {
    /// let cog_driver = DriverManager::get_driver_by_name("COG")?;
    /// println!("{}", cog_driver.long_name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// Cloud optimized GeoTIFF generator
    /// ```
    pub fn get_driver_by_name(name: &str) -> Result<Driver> {
        _register_drivers();
        let c_name = CString::new(name)?;
        let c_driver = unsafe { gdal_sys::GDALGetDriverByName(c_name.as_ptr()) };
        if c_driver.is_null() {
            // `GDALGetDriverByName` just returns `null` and sets no error message
            return Err(GdalError::NullPointer {
                method_name: "GDALGetDriverByName",
                msg: "Unable to find driver".to_string(),
            });
        };
        Ok(Driver { c_driver })
    }

    /// Get a Driver based on the file extension from filename
    ///
    /// Searches for the available extensions in the registered
    /// drivers and returns the matches. The determined driver is
    /// checked for writing capabilities as
    /// [`Dataset::open`](Dataset::open) can already open datasets
    /// with just path.
    ///
    /// See also: [`get_driver_by_name`](Self::get_driver_by_name)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{DriverManager,DriverProperties};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let compatible_driver =
    ///     DriverManager::guess_driver_for_write("test.gpkg", DriverProperties::Vector).unwrap();
    /// println!("{}", compatible_driver.short_name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// "GPKG"
    /// ```
    pub fn guess_driver_for_write<P: AsRef<Path>>(
        filepath: P,
        properties: DriverProperties,
    ) -> Option<Driver> {
        let mut drivers = Self::guess_drivers_for_write(filepath, properties);
        drivers
            .next()
            .map(|d| match d.short_name().to_uppercase().as_str() {
                "GMT" => drivers
                    .find(|d| d.short_name().to_lowercase() == "netcdf")
                    .unwrap_or(d),

                "COG" => drivers
                    .find(|d| d.short_name().to_lowercase() == "gtiff")
                    .unwrap_or(d),
                _ => d,
            })
    }
    /// Get the supported Drivers based on the file extension from filename
    ///
    /// Searches for the available extensions in the registered
    /// drivers and returns the matches. The determined driver is
    /// checked for writing capabilities as
    /// [`Dataset::open`](Dataset::open) can already open datasets
    /// with just path.
    ///
    /// See also: [`get_driver_by_name`](Self::get_driver_by_name)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{DriverManager,DriverProperties};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let compatible_drivers =
    ///     DriverManager::guess_drivers_for_write("test.gpkg", DriverProperties::Vector)
    ///         .map(|d| d.short_name())
    ///         .collect::<Vec<String>>();
    /// println!("{:?}", compatible_drivers);
    /// # Ok(())
    /// # }
    /// ```
    /// ```text
    /// ["GPKG"]
    /// ```
    pub fn guess_drivers_for_write<P: AsRef<Path>>(
        filepath: P,
        properties: DriverProperties,
    ) -> impl Iterator<Item = Driver> {
        let ext = {
            let ext = filepath
                .as_ref()
                .extension()
                .map(|e| e.to_string_lossy().to_string().to_ascii_lowercase());
            match ext.as_deref() {
                Some("zip") => {
                    // zip files could be zipped shp or gpkg
                    let iext = filepath
                        .as_ref()
                        .with_extension("")
                        .extension()
                        .map(|f| f.to_string_lossy().to_string());
                    match iext.as_deref() {
                        Some("shp") => "shp.zip",
                        Some("gpkg") => "gpkg.zip",
                        _ => "zip",
                    }
                }
                Some(e) => e, // normal file with ext
                None => "",
            }
            .to_string()
        };

        DriverManager::all()
            .filter(move |d| {
                let can_create = d.metadata_item("DCAP_CREATE", "").is_some()
                    || d.metadata_item("DCAP_CREATECOPY", "").is_some();
                match properties {
                    DriverProperties::Raster => {
                        can_create && d.metadata_item("DCAP_RASTER", "").is_some()
                    }
                    DriverProperties::Vector => {
                        (can_create && d.metadata_item("DCAP_VECTOR", "").is_some())
                            || d.metadata_item("DCAP_VECTOR_TRANSLATE_FROM", "").is_some()
                    }
                }
            })
            .filter(move |d| {
                if let Some(e) = &d.metadata_item("DMD_EXTENSION", "") {
                    if *e == ext {
                        return true;
                    }
                }
                if let Some(e) = d.metadata_item("DMD_EXTENSIONS", "") {
                    if e.split(' ').any(|s| s == ext) {
                        return true;
                    }
                }

                if let Some(pre) = d.metadata_item("DMD_CONNECTION_PREFIX", "") {
                    if filepath.as_ref().to_string_lossy().starts_with(&pre) {
                        return true;
                    }
                }
                false
            })
    }

    /// Register a driver for use.
    ///
    /// Wraps [`GDALRegisterDriver()`](https://gdal.org/api/raster_c_api.html#_CPPv418GDALRegisterDriver11GDALDriverH)
    pub fn register_driver(driver: &Driver) -> usize {
        let index = unsafe { gdal_sys::GDALRegisterDriver(driver.c_driver) };
        index
            .try_into()
            .expect("The returned index should be zero or positive")
    }

    /// Deregister the passed driver.
    ///
    /// Wraps [`GDALDeregisterDriver()`](https://gdal.org/api/raster_c_api.html#_CPPv420GDALDeregisterDriver11GDALDriverH)
    pub fn deregister_driver(driver: &Driver) {
        unsafe {
            gdal_sys::GDALDeregisterDriver(driver.c_driver);
        }
    }

    /// Register all known GDAL drivers.
    ///
    /// Wraps [`GDALAllRegister()`](https://gdal.org/api/raster_c_api.html#gdal_8h_1a9d40bc998bd6ed07ccde96028e85ae26)
    pub fn register_all() {
        unsafe {
            gdal_sys::GDALAllRegister();
        }
    }

    /// Prevents the automatic registration of all known GDAL drivers when first calling create, open, etc.
    pub fn prevent_auto_registration() {
        START.call_once(|| {});
    }

    /// Destroys the driver manager, i.e., unloads all drivers.
    ///
    /// Wraps [`GDALDestroyDriverManager()`](https://gdal.org/api/raster_c_api.html#_CPPv417GDALDestroyDriver11GDALDriverH)
    pub fn destroy() {
        unsafe {
            gdal_sys::GDALDestroyDriverManager();
        }
    }

    /// Get an `Iterator` over for all the loaded drivers.
    ///
    /// Warning: Adding or removing drivers while consuming the
    /// iterator is safe, but can produce less useful results.
    pub fn all() -> DriverIterator {
        DriverIterator { current: 0 }
    }
}

pub enum DriverProperties {
    Vector,
    Raster,
}

/// Iterator for the registered [`Driver`]s in [`DriverManager`]
pub struct DriverIterator {
    current: usize,
}

impl Iterator for DriverIterator {
    type Item = Driver;

    fn next(&mut self) -> Option<Self::Item> {
        match DriverManager::get_driver(self.current) {
            Ok(d) => {
                self.current += 1;
                Some(d)
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_driver_access() {
        let driver = DriverManager::get_driver_by_name("GTiff").unwrap();
        assert_eq!(driver.short_name(), "GTiff");
        assert_eq!(driver.long_name(), "GeoTIFF");

        assert!(DriverManager::count() > 0);
        assert!(DriverManager::get_driver(0).is_ok());
    }

    #[test]
    fn test_driver_by_extension() {
        fn test_driver(d: &Driver, filename: &str, properties: DriverProperties) {
            assert_eq!(
                DriverManager::guess_driver_for_write(filename, properties)
                    .unwrap()
                    .short_name(),
                d.short_name()
            );
        }

        if let Ok(d) = DriverManager::get_driver_by_name("ESRI Shapefile") {
            test_driver(&d, "test.shp", DriverProperties::Vector);
            test_driver(&d, "my.test.shp", DriverProperties::Vector);
            // `shp.zip` only supported from gdal version 3.1
            // https://gdal.org/drivers/vector/shapefile.html#compressed-files
            if cfg!(all(major_ge_3, minor_ge_1)) {
                test_driver(&d, "test.shp.zip", DriverProperties::Vector);
                test_driver(&d, "my.test.shp.zip", DriverProperties::Vector);
            }
        }

        if let Ok(d) = DriverManager::get_driver_by_name("GTiff") {
            test_driver(&d, "test.tiff", DriverProperties::Raster);
            test_driver(&d, "my.test.tiff", DriverProperties::Raster);
        }
        if let Ok(d) = DriverManager::get_driver_by_name("netCDF") {
            test_driver(&d, "test.nc", DriverProperties::Raster);
        }
    }

    #[test]
    fn test_drivers_by_extension() {
        // convert the driver into short_name for testing purposes
        let drivers = |filename, is_vector| {
            DriverManager::guess_drivers_for_write(
                filename,
                if is_vector {
                    DriverProperties::Vector
                } else {
                    DriverProperties::Raster
                },
            )
            .map(|d| d.short_name())
            .collect::<HashSet<String>>()
        };
        if DriverManager::get_driver_by_name("ESRI Shapefile").is_ok() {
            assert!(drivers("test.shp", true).contains("ESRI Shapefile"));
            assert!(drivers("my.test.shp", true).contains("ESRI Shapefile"));
            // `shp.zip` only supported from gdal version 3.1
            // https://gdal.org/drivers/vector/shapefile.html#compressed-files
            if cfg!(all(major_ge_3, minor_ge_1)) {
                assert!(drivers("test.shp.zip", true).contains("ESRI Shapefile"));
                assert!(drivers("my.test.shp.zip", true).contains("ESRI Shapefile"));
            }
        }
        if DriverManager::get_driver_by_name("GPKG").is_ok() {
            assert!(drivers("test.gpkg", true).contains("GPKG"));
            assert!(drivers("my.test.gpkg", true).contains("GPKG"));
            // `gpkg.zip` only supported from gdal version 3.7
            // https://gdal.org/drivers/vector/gpkg.html#compressed-files
            if cfg!(all(major_ge_3, minor_ge_7)) {
                assert!(drivers("test.gpkg.zip", true).contains("GPKG"));
                assert!(drivers("my.test.gpkg.zip", true).contains("GPKG"));
            }
        }
        if DriverManager::get_driver_by_name("GTiff").is_ok() {
            assert!(drivers("test.tiff", false).contains("GTiff"));
            assert!(drivers("my.test.tiff", false).contains("GTiff"));
        }
        if DriverManager::get_driver_by_name("netCDF").is_ok() {
            assert!(drivers("test.nc", false).contains("netCDF"));
        }
        if DriverManager::get_driver_by_name("PostgreSQL").is_ok() {
            assert!(drivers("PG:test", true).contains("PostgreSQL"));
        }
    }

    #[test]
    fn test_driver_iterator() {
        assert_eq!(DriverManager::count(), DriverManager::all().count());

        let drivers: HashSet<String> = DriverManager::all().map(|d| d.short_name()).collect();
        for i in 0..DriverManager::count() {
            assert!(drivers.contains(&DriverManager::get_driver(i).unwrap().short_name()))
        }
    }
}
