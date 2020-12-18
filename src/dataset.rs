use std::{ffi::CString, ffi::NulError, path::Path, ptr, sync::Once};

use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use crate::{
    gdal_major_object::MajorObject, raster::RasterBand, spatial_ref::SpatialRef, vector::Layer,
    Driver, Metadata,
};
use gdal_sys::{
    self, CPLErr, GDALAccess, GDALDatasetH, GDALMajorObjectH, OGRErr, OGRLayerH, OGRwkbGeometryType,
};
use libc::{c_double, c_int};
use ptr::null_mut;

use crate::errors::*;
use std::convert::TryInto;

pub type GeoTransform = [c_double; 6];
static START: Once = Once::new();

#[derive(Debug)]
pub struct Dataset {
    c_dataset: GDALDatasetH,
}

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::GDALAllRegister();
        });
    }
}

// GDAL Docs state: The returned dataset should only be accessed by one thread at a time.
// See: https://gdal.org/api/raster_c_api.html#_CPPv48GDALOpenPKc10GDALAccess
// Additionally, VRT Datasets are not safe before GDAL 2.3.
// See: https://gdal.org/drivers/raster/vrt.html#multi-threading-issues
#[cfg(any(all(major_is_2, minor_ge_3), major_ge_3))]
unsafe impl Send for Dataset {}

impl Dataset {
    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_dataset(&self) -> GDALDatasetH {
        self.c_dataset
    }

    pub fn open(path: &Path) -> Result<Dataset> {
        Self::open_ex(path, None, None, None, None)
    }

    pub fn open_ex(
        path: &Path,
        open_flags: Option<GDALAccess::Type>,
        allowed_drivers: Option<&[&str]>, // TODO: use parameters
        open_options: Option<&[&str]>,
        sibling_files: Option<&[&str]>,
    ) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_open_flags = open_flags.unwrap_or(GDALAccess::GA_ReadOnly); // This defaults to GdalAccess::GA_ReadOnly

        // handle driver params:
        // we need to keep the CStrings and the pointers around
        let c_allowed_drivers = allowed_drivers.map(|d| {
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

        let c_drivers_ptr = if allowed_drivers.is_some() {
            c_drivers_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        // handle open options params:
        // we need to keep the CStrings and the pointers around
        let c_open_options = open_options.map(|d| {
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

        let c_open_options_ptr = if open_options.is_some() {
            c_open_options_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        // handle sibling files params:
        // we need to keep the CStrings and the pointers around
        let c_sibling_files = sibling_files.map(|d| {
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

        let c_sibling_files_ptr = if sibling_files.is_some() {
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
        Ok(Dataset { c_dataset })
    }

    /// Creates a new Dataset by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_dataset(c_dataset: GDALDatasetH) -> Dataset {
        Dataset { c_dataset }
    }

    pub fn projection(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetProjectionRef(self.c_dataset) };
        _string(rv)
    }

    pub fn set_projection(&self, projection: &str) -> Result<()> {
        let c_projection = CString::new(projection)?;
        unsafe { gdal_sys::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
        Ok(())
    }

    #[cfg(major_ge_3)]
    pub fn spatial_ref(&self) -> Result<SpatialRef> {
        unsafe { SpatialRef::from_c_obj(gdal_sys::GDALGetSpatialRef(self.c_dataset)) }
    }

    #[cfg(major_ge_3)]
    pub fn set_spatial_ref(&self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetSpatialRef(self.c_dataset, spatial_ref.to_c_hsrs()) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    pub fn create_copy(&self, driver: &Driver, filename: &str) -> Result<Dataset> {
        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreateCopy(
                driver.c_driver(),
                c_filename.as_ptr(),
                self.c_dataset,
                0,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALCreateCopy"));
        }
        Ok(unsafe { Dataset::from_c_dataset(c_dataset) })
    }

    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal_sys::GDALGetDatasetDriver(self.c_dataset);
            Driver::from_c_driver(c_driver)
        }
    }

    pub fn rasterband(&self, band_index: isize) -> Result<RasterBand> {
        unsafe {
            let c_band = gdal_sys::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            if c_band.is_null() {
                return Err(_last_null_pointer_err("GDALGetRasterBand"));
            }
            Ok(RasterBand::from_c_rasterband(self, c_band))
        }
    }

    fn child_layer(&self, c_layer: OGRLayerH) -> Layer {
        unsafe { Layer::from_c_layer(self, c_layer) }
    }

    pub fn layer_count(&self) -> isize {
        (unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset) }) as isize
    }

    pub fn layer(&mut self, idx: isize) -> Result<Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer"));
        }
        Ok(self.child_layer(c_layer))
    }

    pub fn layer_by_name(&mut self, name: &str) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName"));
        }
        Ok(self.child_layer(c_layer))
    }

    pub fn layers(&self) -> LayerIterator {
        LayerIterator::with_dataset(self)
    }

    pub fn raster_count(&self) -> isize {
        (unsafe { gdal_sys::GDALGetRasterCount(self.c_dataset) }) as isize
    }

    pub fn raster_size(&self) -> (usize, usize) {
        let size_x = unsafe { gdal_sys::GDALGetRasterXSize(self.c_dataset) } as usize;
        let size_y = unsafe { gdal_sys::GDALGetRasterYSize(self.c_dataset) } as usize;
        (size_x, size_y)
    }

    /// Create a new layer with a blank name, no `SpatialRef`, and without (wkbUnknown) geometry type.
    pub fn create_layer_blank(&mut self) -> Result<Layer> {
        self.create_layer("", None, OGRwkbGeometryType::wkbUnknown)
    }

    /// Create a new layer with a name, an optional `SpatialRef`, and a geometry type.
    pub fn create_layer(
        &mut self,
        name: &str,
        srs: Option<&SpatialRef>,
        ty: OGRwkbGeometryType::Type,
    ) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_srs = match srs {
            Some(srs) => srs.to_c_hsrs(),
            None => null_mut(),
        };

        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(self.c_dataset, c_name.as_ptr(), c_srs, ty, null_mut())
        };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_CreateLayer"));
        };
        Ok(self.child_layer(c_layer))
    }

    /// Affine transformation called geotransformation.
    ///
    /// This is like a linear transformation preserves points, straight lines and planes.
    /// Also, sets of parallel lines remain parallel after an affine transformation.
    /// # Arguments
    /// * transformation - coeficients of transformations
    ///
    /// x-coordinate of the top-left corner pixel (x-offset)
    /// width of a pixel (x-resolution)
    /// row rotation (typically zero)
    /// y-coordinate of the top-left corner pixel
    /// column rotation (typically zero)
    /// height of a pixel (y-resolution, typically negative)
    pub fn set_geo_transform(&self, transformation: &GeoTransform) -> Result<()> {
        assert_eq!(transformation.len(), 6);
        let rv = unsafe {
            gdal_sys::GDALSetGeoTransform(self.c_dataset, transformation.as_ptr() as *mut f64)
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Get affine transformation coefficients.
    ///
    /// x-coordinate of the top-left corner pixel (x-offset)
    /// width of a pixel (x-resolution)
    /// row rotation (typically zero)
    /// y-coordinate of the top-left corner pixel
    /// column rotation (typically zero)
    /// height of a pixel (y-resolution, typically negative)
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

    /// For datasources which support transactions, this creates a transaction.
    ///
    /// During the transaction, the dataset can be mutably borrowed using
    /// [`Transaction::dataset_mut`] to make changes. All changes done after the start of the
    /// transaction are applied to the datasource when [`commit`](Transaction::commit) is called.
    /// They may be canceled by calling [`rollback`](Transaction::rollback) instead, or by dropping
    /// the `Transaction` without calling `commit`.
    ///
    /// Depending on the driver, using a transaction can give a huge performance improvement when
    /// creating a lot of geometry at once. This is because the driver doesn't need to commit every
    /// feature to disk individually.
    ///
    /// If starting the transaction fails, this function will return [`OGRErr::OGRERR_FAILURE`].
    /// For datasources that do not support transactions, this function will always return
    /// [`OGRErr::OGRERR_UNSUPPORTED_OPERATION`].
    ///
    /// Limitations:
    ///
    /// * Datasources which do not support efficient transactions natively may use less efficient
    ///   emulation of transactions instead; as of GDAL 3.1, this only applies to the closed-source
    ///   FileGDB driver, which (unlike OpenFileGDB) is not available in a GDAL build by default.
    ///
    /// * At the time of writing, transactions only apply on vector layers.
    ///
    /// * Nested transactions are not supported.
    ///
    /// * If an error occurs after a successful `start_transaction`, the whole transaction may or
    ///   may not be implicitly canceled, depending on the driver. For example, the PG driver will
    ///   cancel it, but the SQLite and GPKG drivers will not.
    ///
    /// Example:
    ///
    /// ```
    /// # use gdal::Dataset;
    /// #
    /// fn create_point_grid(dataset: &mut Dataset) -> gdal::errors::Result<()> {
    ///     use gdal::vector::Geometry;
    ///
    ///     // Start the transaction.
    ///     let mut txn = dataset.start_transaction()?;
    ///
    ///     let mut layer = txn.dataset_mut()
    ///         .create_layer("grid", None, gdal_sys::OGRwkbGeometryType::wkbPoint)?;
    ///     for y in 0..100 {
    ///         for x in 0..100 {
    ///             let wkt = format!("POINT ({} {})", x, y);
    ///             layer.create_feature(Geometry::from_wkt(&wkt)?)?;
    ///         }
    ///     }
    ///
    ///     // We got through without errors. Commit the transaction and return.
    ///     txn.commit()?;
    ///     Ok(())
    /// }
    /// #
    /// # fn main() -> gdal::errors::Result<()> {
    /// #     let driver = gdal::Driver::get("SQLite")?;
    /// #     let mut dataset = driver.create_vector_only(":memory:")?;
    /// #     create_point_grid(&mut dataset)?;
    /// #     assert_eq!(dataset.layer(0)?.features().count(), 10000);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn start_transaction(&mut self) -> Result<Transaction<'_>> {
        let force = 1;
        let rv = unsafe { gdal_sys::GDALDatasetStartTransaction(self.c_dataset, force) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetStartTransaction",
            });
        }
        Ok(Transaction::new(self))
    }
}

pub struct LayerIterator<'a> {
    dataset: &'a Dataset,
    idx: isize,
    count: isize
}

impl<'a> Iterator for LayerIterator<'a> {
    type Item = Layer<'a>;

    #[inline]
    fn next(&mut self) -> Option<Layer<'a>> {
        let idx = self.idx;
        if idx < self.count {
            self.idx += 1;
            let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.dataset.c_dataset, idx as c_int) };
            if !c_layer.is_null() {
                let layer = unsafe { Layer::from_c_layer(self.dataset, c_layer) };
                return Some(layer);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match Some(self.count).map(|s| s.try_into().ok()).flatten() {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl<'a> LayerIterator<'a> {
    pub fn with_dataset(dataset: &'a Dataset) -> LayerIterator<'a> {
        LayerIterator { dataset, idx: 0, count: dataset.layer_count() }
    }
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALClose(self.c_dataset);
        }
    }
}

/// Represents an in-flight transaction on a dataset.
///
/// It can either be committed by calling [`commit`](Transaction::commit) or rolled back by calling
/// [`rollback`](Transaction::rollback).
///
/// If the transaction is not explicitly committed when it is dropped, it is implicitly rolled
/// back.
///
/// The transaction holds a mutable borrow on the `Dataset` that it was created from, so during the
/// lifetime of the transaction you will need to access the dataset through
/// [`Transaction::dataset`] or [`Transaction::dataset_mut`].
#[derive(Debug)]
pub struct Transaction<'a> {
    dataset: &'a mut Dataset,
    rollback_on_drop: bool,
}

impl<'a> Transaction<'a> {
    fn new(dataset: &'a mut Dataset) -> Self {
        Transaction {
            dataset,
            rollback_on_drop: true,
        }
    }

    /// Returns a reference to the dataset from which this `Transaction` was created.
    pub fn dataset(&self) -> &Dataset {
        self.dataset
    }

    /// Returns a mutable reference to the dataset from which this `Transaction` was created.
    pub fn dataset_mut(&mut self) -> &mut Dataset {
        self.dataset
    }

    /// Commits this transaction.
    ///
    /// If the commit fails, will return [`OGRErr::OGRERR_FAILURE`].
    ///
    /// Depending on drivers, this may or may not abort layer sequential readings that are active.
    pub fn commit(mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALDatasetCommitTransaction(self.dataset.c_dataset) };
        self.rollback_on_drop = false;
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetCommitTransaction",
            });
        }
        Ok(())
    }

    /// Rolls back the dataset to its state before the start of this transaction.
    ///
    /// If the rollback fails, will return [`OGRErr::OGRERR_FAILURE`].
    pub fn rollback(mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALDatasetRollbackTransaction(self.dataset.c_dataset) };
        self.rollback_on_drop = false;
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetRollbackTransaction",
            });
        }
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.rollback_on_drop {
            // We silently swallow any errors, because we have no way to report them from a drop
            // function apart from panicking.
            unsafe { gdal_sys::GDALDatasetRollbackTransaction(self.dataset.c_dataset) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::Geometry;
    use tempfile::TempPath;

    macro_rules! fixture {
        ($name:expr) => {
            Path::new(file!())
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("fixtures")
                .as_path()
                .join($name)
                .as_path()
        };
    }

    /// Copies the given file to a temporary file and opens it for writing. When the returned
    /// `TempPath` is dropped, the file is deleted.
    fn open_gpkg_for_update(path: &Path) -> (TempPath, Dataset) {
        use std::fs;
        use std::io::Write;

        let input_data = fs::read(path).unwrap();
        let (mut file, temp_path) = tempfile::Builder::new()
            .suffix(".gpkg")
            .tempfile()
            .unwrap()
            .into_parts();
        file.write_all(&input_data).unwrap();
        // Close the temporary file so that Dataset can open it safely even if the filesystem uses
        // exclusive locking (Windows?).
        drop(file);

        let ds = Dataset::open_ex(
            &temp_path,
            Some(GDALAccess::GA_Update),
            Some(&["GPKG"]),
            None,
            None,
        )
        .unwrap();
        (temp_path, ds)
    }

    fn polygon() -> Geometry {
        Geometry::from_wkt("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))").unwrap()
    }

    #[test]
    fn test_open_vector() {
        Dataset::open(fixture!("roads.geojson")).unwrap();
    }

    #[test]
    fn test_open_ex_ro_vector() {
        Dataset::open_ex(
            fixture!("roads.geojson"),
            Some(GDALAccess::GA_ReadOnly),
            None,
            None,
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_update_vector() {
        Dataset::open_ex(
            fixture!("roads.geojson"),
            Some(GDALAccess::GA_Update),
            None,
            None,
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_allowed_driver_vector() {
        Dataset::open_ex(
            fixture!("roads.geojson"),
            None,
            Some(&["GeoJSON"]),
            None,
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_open_ex_allowed_driver_vector_fail() {
        Dataset::open_ex(fixture!("roads.geojson"), None, Some(&["TIFF"]), None, None).unwrap_err();
    }

    #[test]
    fn test_open_ex_open_option() {
        Dataset::open_ex(
            fixture!("roads.geojson"),
            None,
            None,
            Some(&["FLATTEN_NESTED_ATTRIBUTES=YES"]),
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_layer_count() {
        let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
        assert_eq!(ds.layer_count(), 1);
    }

    #[test]
    fn test_raster_count_on_vector() {
        let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
        assert_eq!(ds.raster_count(), 0);
    }

    #[test]
    fn test_start_transaction() {
        let (_temp_path, mut ds) = open_gpkg_for_update(fixture!("poly.gpkg"));
        let txn = ds.start_transaction();
        assert!(txn.is_ok());
    }

    #[test]
    fn test_transaction_commit() {
        let (_temp_path, mut ds) = open_gpkg_for_update(fixture!("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let mut txn = ds.start_transaction().unwrap();
        let mut layer = txn.dataset_mut().layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.commit().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count + 1);
    }

    #[test]
    fn test_transaction_rollback() {
        let (_temp_path, mut ds) = open_gpkg_for_update(fixture!("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let mut txn = ds.start_transaction().unwrap();
        let mut layer = txn.dataset_mut().layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.rollback().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count);
    }

    #[test]
    fn test_transaction_implicit_rollback() {
        let (_temp_path, mut ds) = open_gpkg_for_update(fixture!("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        {
            let mut txn = ds.start_transaction().unwrap();
            let mut layer = txn.dataset_mut().layer(0).unwrap();
            layer.create_feature(polygon()).unwrap();
        } // txn is dropped here.

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count);
    }

    #[test]
    fn test_start_transaction_unsupported() {
        let mut ds = Dataset::open(fixture!("roads.geojson")).unwrap();
        assert!(ds.start_transaction().is_err());
    }
}
