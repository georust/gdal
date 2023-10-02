use foreign_types::ForeignTypeRef;
use std::{ffi::CString, ffi::NulError, path::Path, ptr};

use gdal_sys::{self, CPLErr, GDALDatasetH, GDALMajorObjectH};

use crate::cpl::CslStringList;
use crate::errors::*;
use crate::options::DatasetOptions;
use crate::raster::RasterCreationOption;
use crate::spatial_ref::SpatialRefRef;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string, _string};
use crate::{
    gdal_major_object::MajorObject, spatial_ref::SpatialRef, Driver, GeoTransform, Metadata,
};

/// Wrapper around a [`GDALDataset`][GDALDataset] object.
///
/// Represents both a [vector dataset][vector-data-model]
/// containing a collection of layers; and a
/// [raster dataset][raster-data-model] containing a collection of raster-bands.
///
/// [vector-data-model]: https://gdal.org/user/vector_data_model.html
/// [raster-data-model]: https://gdal.org/user/raster_data_model.html
/// [GDALDataset]: https://gdal.org/api/gdaldataset_cpp.html#_CPPv411GDALDataset
#[derive(Debug)]
pub struct Dataset {
    c_dataset: GDALDatasetH,
    closed: bool,
}

// GDAL Docs state: The returned dataset should only be accessed by one thread at a time.
// See: https://gdal.org/api/raster_c_api.html#_CPPv48GDALOpenPKc10GDALAccess
// Additionally, VRT Datasets are not safe before GDAL 2.3.
// See: https://gdal.org/drivers/raster/vrt.html#multi-threading-issues
#[cfg(any(all(major_is_2, minor_ge_3), major_ge_3))]
unsafe impl Send for Dataset {}

/// Core dataset methods
impl Dataset {
    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub fn c_dataset(&self) -> GDALDatasetH {
        self.c_dataset
    }

    /// Creates a new Dataset by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    /// The dataset must not have been closed (using [`GDALClose`]) before.
    pub unsafe fn from_c_dataset(c_dataset: GDALDatasetH) -> Dataset {
        Dataset {
            c_dataset,
            closed: false,
        }
    }

    /// Open a dataset at the given `path` with default
    /// options.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Dataset> {
        Self::_open_ex(path.as_ref(), DatasetOptions::default())
    }

    /// Open a dataset with extended options. See
    /// [`GDALOpenEx`].
    ///
    /// [`GDALOpenEx`]: https://gdal.org/doxygen/gdal_8h.html#a9cb8585d0b3c16726b08e25bcc94274a
    pub fn open_ex<P: AsRef<Path>>(path: P, options: DatasetOptions) -> Result<Dataset> {
        Self::_open_ex(path.as_ref(), options)
    }

    fn _open_ex(path: &Path, options: DatasetOptions) -> Result<Dataset> {
        crate::driver::_register_drivers();

        let c_filename = _path_to_c_string(path)?;
        let c_open_flags = options.open_flags.bits();

        // handle driver params:
        // we need to keep the CStrings and the pointers around
        let c_allowed_drivers = options.allowed_drivers.map(|d| {
            d.iter()
                .map(|&s| CString::new(s))
                .collect::<std::result::Result<Vec<CString>, NulError>>()
        });
        let c_drivers_vec = match c_allowed_drivers {
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(c_drivers_vec)) => c_drivers_vec,
            None => Vec::from([]),
        };
        let mut c_drivers_ptrs = c_drivers_vec.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        c_drivers_ptrs.push(ptr::null());

        let c_drivers_ptr = if options.allowed_drivers.is_some() {
            c_drivers_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        // handle open options params:
        // we need to keep the CStrings and the pointers around
        let c_open_options = options.open_options.map(|d| {
            d.iter()
                .map(|&s| CString::new(s))
                .collect::<std::result::Result<Vec<CString>, NulError>>()
        });
        let c_open_options_vec = match c_open_options {
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(c_open_options_vec)) => c_open_options_vec,
            None => Vec::from([]),
        };
        let mut c_open_options_ptrs = c_open_options_vec
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();
        c_open_options_ptrs.push(ptr::null());

        let c_open_options_ptr = if options.open_options.is_some() {
            c_open_options_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        // handle sibling files params:
        // we need to keep the CStrings and the pointers around
        let c_sibling_files = options.sibling_files.map(|d| {
            d.iter()
                .map(|&s| CString::new(s))
                .collect::<std::result::Result<Vec<CString>, NulError>>()
        });
        let c_sibling_files_vec = match c_sibling_files {
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(c_sibling_files_vec)) => c_sibling_files_vec,
            None => Vec::from([]),
        };
        let mut c_sibling_files_ptrs = c_sibling_files_vec
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();
        c_sibling_files_ptrs.push(ptr::null());

        let c_sibling_files_ptr = if options.sibling_files.is_some() {
            c_sibling_files_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        let c_dataset = unsafe {
            gdal_sys::GDALOpenEx(
                c_filename.as_ptr(),
                c_open_flags,
                c_drivers_ptr,
                c_open_options_ptr,
                c_sibling_files_ptr,
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALOpenEx"));
        }
        Ok(Dataset {
            c_dataset,
            closed: false,
        })
    }

    /// Flush all write cached data to disk.
    ///
    /// See [`GDALFlushCache`].
    ///
    /// Note: on GDAL versions older than 3.7, this function always succeeds.
    pub fn flush_cache(&mut self) -> Result<()> {
        #[cfg(any(all(major_ge_3, minor_ge_7), major_ge_4))]
        {
            let rv = unsafe { gdal_sys::GDALFlushCache(self.c_dataset) };
            if rv != CPLErr::CE_None {
                return Err(_last_cpl_err(rv));
            }
        }
        #[cfg(not(any(all(major_is_3, minor_ge_7), major_ge_4)))]
        {
            unsafe {
                gdal_sys::GDALFlushCache(self.c_dataset);
            }
        }
        Ok(())
    }

    /// Close the dataset.
    ///
    /// See [`GDALClose`].
    ///
    /// Note: on GDAL versions older than 3.7.0, this function always succeeds.
    pub fn close(mut self) -> Result<()> {
        self.closed = true;

        #[cfg(any(all(major_ge_3, minor_ge_7), major_ge_4))]
        {
            let rv = unsafe { gdal_sys::GDALClose(self.c_dataset) };
            if rv != CPLErr::CE_None {
                return Err(_last_cpl_err(rv));
            }
        }
        #[cfg(not(any(all(major_is_3, minor_ge_7), major_ge_4)))]
        {
            unsafe {
                gdal_sys::GDALClose(self.c_dataset);
            }
        }
        Ok(())
    }

    /// Fetch the projection definition string for this dataset.
    pub fn projection(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetProjectionRef(self.c_dataset) };
        _string(rv)
    }

    /// Set the projection reference string for this dataset.
    pub fn set_projection(&mut self, projection: &str) -> Result<()> {
        let c_projection = CString::new(projection)?;
        unsafe { gdal_sys::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
        Ok(())
    }

    #[cfg(major_ge_3)]
    /// Get the spatial reference system for this dataset.
    pub fn spatial_ref(&self) -> Result<SpatialRef> {
        Ok(
            unsafe { SpatialRefRef::from_ptr(gdal_sys::GDALGetSpatialRef(self.c_dataset)) }
                .to_owned(),
        )
    }

    #[cfg(major_ge_3)]
    /// Set the spatial reference system for this dataset.
    pub fn set_spatial_ref(&mut self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetSpatialRef(self.c_dataset, spatial_ref.as_ptr()) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    pub fn create_copy<P: AsRef<Path>>(
        &self,
        driver: &Driver,
        filename: P,
        options: &[RasterCreationOption],
    ) -> Result<Dataset> {
        fn _create_copy(
            ds: &Dataset,
            driver: &Driver,
            filename: &Path,
            options: &[RasterCreationOption],
        ) -> Result<Dataset> {
            let c_filename = _path_to_c_string(filename)?;

            let mut c_options = CslStringList::new();
            for option in options {
                c_options.set_name_value(option.key, option.value)?;
            }

            let c_dataset = unsafe {
                gdal_sys::GDALCreateCopy(
                    driver.c_driver(),
                    c_filename.as_ptr(),
                    ds.c_dataset,
                    0,
                    c_options.as_ptr(),
                    None,
                    ptr::null_mut(),
                )
            };
            if c_dataset.is_null() {
                return Err(_last_null_pointer_err("GDALCreateCopy"));
            }
            Ok(unsafe { Dataset::from_c_dataset(c_dataset) })
        }
        _create_copy(self, driver, filename.as_ref(), options)
    }

    /// Fetch the driver to which this dataset relates.
    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal_sys::GDALGetDatasetDriver(self.c_dataset);
            Driver::from_c_driver(c_driver)
        }
    }

    /// Set the [`Dataset`]'s affine transformation; also called a _geo-transformation_.
    ///
    /// This is like a linear transformation preserves points, straight lines and planes.
    /// Also, sets of parallel lines remain parallel after an affine transformation.
    ///
    /// # Arguments
    /// * `transformation` - coefficients of the transformation, which are:
    ///    - x-coordinate of the top-left corner pixel (x-offset)
    ///    - width of a pixel (x-resolution)
    ///    - row rotation (typically zero)
    ///    - y-coordinate of the top-left corner pixel
    ///    - column rotation (typically zero)
    ///    - height of a pixel (y-resolution, typically negative)
    pub fn set_geo_transform(&mut self, transformation: &GeoTransform) -> Result<()> {
        assert_eq!(transformation.len(), 6);
        let rv = unsafe {
            gdal_sys::GDALSetGeoTransform(self.c_dataset, transformation.as_ptr() as *mut f64)
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Get the coefficients of the [`Dataset`]'s affine transformation.
    ///
    /// # Returns
    /// - x-coordinate of the top-left corner pixel (x-offset)
    /// - width of a pixel (x-resolution)
    /// - row rotation (typically zero)
    /// - y-coordinate of the top-left corner pixel
    /// - column rotation (typically zero)
    /// - height of a pixel (y-resolution, typically negative)
    pub fn geo_transform(&self) -> Result<GeoTransform> {
        let mut transformation = GeoTransform::default();
        let rv =
            unsafe { gdal_sys::GDALGetGeoTransform(self.c_dataset, transformation.as_mut_ptr()) };

        // check if the dataset has a GeoTransform
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(transformation)
    }
}

impl MajorObject for Dataset {
    fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Drop for Dataset {
    fn drop(&mut self) {
        if !self.closed {
            unsafe {
                gdal_sys::GDALClose(self.c_dataset);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gdal_sys::GDALAccess;

    use crate::test_utils::fixture;
    use crate::GdalOpenFlags;

    use super::*;

    #[test]
    fn test_open_vector() {
        let dataset = Dataset::open(fixture("roads.geojson")).unwrap();
        dataset.close().unwrap();
    }

    #[test]
    fn test_open_ex_ro_vector() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                open_flags: GDALAccess::GA_ReadOnly.into(),
                ..DatasetOptions::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_update_vector() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                open_flags: GDALAccess::GA_Update.into(),
                ..DatasetOptions::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_allowed_driver_vector() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                allowed_drivers: Some(&["GeoJSON"]),
                ..DatasetOptions::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_allowed_driver_vector_fail() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                allowed_drivers: Some(&["TIFF"]),
                ..DatasetOptions::default()
            },
        )
        .unwrap_err();
    }

    #[test]
    fn test_open_ex_open_option() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                open_options: Some(&["FLATTEN_NESTED_ATTRIBUTES=YES"]),
                ..DatasetOptions::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_extended_flags_vector() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                open_flags: GdalOpenFlags::GDAL_OF_UPDATE | GdalOpenFlags::GDAL_OF_VECTOR,
                ..DatasetOptions::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_extended_flags_vector_fail() {
        Dataset::open_ex(
            fixture("roads.geojson"),
            DatasetOptions {
                open_flags: GdalOpenFlags::GDAL_OF_UPDATE | GdalOpenFlags::GDAL_OF_RASTER,
                ..DatasetOptions::default()
            },
        )
        .unwrap_err();
    }

    #[test]
    fn test_raster_count_on_vector() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        assert_eq!(ds.raster_count(), 0);
    }
}
