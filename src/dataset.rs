use ptr::null_mut;
use std::convert::TryInto;
use std::mem::MaybeUninit;
use std::{
    ffi::NulError,
    ffi::{CStr, CString},
    path::Path,
    ptr,
};

use crate::cpl::CslStringList;
use crate::errors::*;
use crate::raster::RasterCreationOption;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string, _string};
use crate::vector::{sql, Geometry, OwnedLayer};
use crate::{
    gdal_major_object::MajorObject, raster::RasterBand, spatial_ref::SpatialRef, vector::Layer,
    Driver, Metadata,
};
use gdal_sys::OGRGeometryH;
use gdal_sys::{
    self, CPLErr, GDALAccess, GDALDatasetH, GDALMajorObjectH, OGRErr, OGRLayerH, OGRwkbGeometryType,
};
use libc::{c_double, c_int, c_uint};

use bitflags::bitflags;

/// A 2-D affine transform mapping pixel coordiates to world
/// coordinates. See [GDALGetGeoTransform] for more details.
///
/// [GDALGetGeoTransform]: https://gdal.org/api/gdaldataset_cpp.html#classGDALDataset_1a5101119705f5fa2bc1344ab26f66fd1d
pub type GeoTransform = [c_double; 6];

pub trait GeoTransformEx {
    fn apply(&self, pixel: f64, line: f64) -> (f64, f64);
    fn invert(&self) -> Result<GeoTransform>;
}

impl GeoTransformEx for GeoTransform {
    /// Apply GeoTransform to x/y coordinate.
    /// Wraps [GDALApplyGeoTransform].
    ///
    /// [GDALApplyGeoTransform]: https://gdal.org/api/raster_c_api.html#_CPPv421GDALApplyGeoTransformPdddPdPd
    fn apply(&self, pixel: f64, line: f64) -> (f64, f64) {
        let mut geo_x = MaybeUninit::<f64>::uninit();
        let mut geo_y = MaybeUninit::<f64>::uninit();
        unsafe {
            gdal_sys::GDALApplyGeoTransform(
                self.as_ptr() as *mut f64,
                pixel,
                line,
                geo_x.as_mut_ptr(),
                geo_y.as_mut_ptr(),
            );
            (geo_x.assume_init(), geo_y.assume_init())
        }
    }

    /// Invert Geotransform.
    /// Wraps [GDALInvGeoTransform].
    ///
    /// [GDALInvGeoTransform]: https://gdal.org/api/raster_c_api.html#_CPPv419GDALInvGeoTransformPdPd
    fn invert(&self) -> Result<GeoTransform> {
        let mut gt_out = MaybeUninit::<GeoTransform>::uninit();
        let rv = unsafe {
            gdal_sys::GDALInvGeoTransform(
                self.as_ptr() as *mut f64,
                (&mut *gt_out.as_mut_ptr()).as_mut_ptr(),
            )
        };
        if rv == 0 {
            return Err(GdalError::BadArgument(
                "Geo transform is uninvertible".to_string(),
            ));
        }
        let result = unsafe { gt_out.assume_init() };
        Ok(result)
    }
}

/// Wrapper around a [`GDALDataset`][GDALDataset] object.
///
/// Represents both a [vector dataset][vector-data-model]
/// containing a collection of layers; and a [raster
/// dataset][raster-data-model] containing a collection of
/// rasterbands.
///
/// [vector-data-model]: https://gdal.org/user/vector_data_model.html
/// [raster-data-model]: https://gdal.org/user/raster_data_model.html
/// [GDALDataset]: https://gdal.org/api/gdaldataset_cpp.html#_CPPv411GDALDataset
#[derive(Debug)]
pub struct Dataset {
    c_dataset: GDALDatasetH,
}

// These are skipped by bindgen and manually updated.
#[cfg(major_ge_2)]
bitflags! {
    /// GDal extended open flags used by [`Dataset::open_ex`].
    ///
    /// Used in the `nOpenFlags` argument to [`GDALOpenEx`].
    ///
    /// Note that the `GDAL_OF_SHARED` option is removed
    /// from the set of allowed option because it subverts
    /// the [`Send`] implementation that allow passing the
    /// dataset the another thread. See
    /// https://github.com/georust/gdal/issues/154.
    ///
    /// [`GDALOpenEx`]: https://gdal.org/doxygen/gdal_8h.html#a9cb8585d0b3c16726b08e25bcc94274a
    pub struct GdalOpenFlags: c_uint {
        /// Open in read-only mode (default).
        const GDAL_OF_READONLY = 0x00;
        /// Open in update mode.
        const GDAL_OF_UPDATE = 0x01;
        /// Allow raster and vector drivers to be used.
        const GDAL_OF_ALL = 0x00;
        /// Allow raster drivers to be used.
        const GDAL_OF_RASTER = 0x02;
        /// Allow vector drivers to be used.
        const GDAL_OF_VECTOR = 0x04;
        /// Allow gnm drivers to be used.
        #[cfg(any( all(major_ge_2,minor_ge_1), major_ge_3 ))]
        const GDAL_OF_GNM = 0x08;
        /// Allow multidimensional raster drivers to be used.
        #[cfg(all(major_ge_3,minor_ge_1))]
        const GDAL_OF_MULTIDIM_RASTER = 0x10;
        /// Emit error message in case of failed open.
        const GDAL_OF_VERBOSE_ERROR = 0x40;
        /// Open as internal dataset. Such dataset isn't
        /// registered in the global list of opened dataset.
        /// Cannot be used with GDAL_OF_SHARED.
        const GDAL_OF_INTERNAL = 0x80;

        /// Default strategy for cached blocks.
        #[cfg(any( all(major_ge_2,minor_ge_1), major_ge_3 ))]
        const GDAL_OF_DEFAULT_BLOCK_ACCESS = 0;

        /// Array based strategy for cached blocks.
        #[cfg(any( all(major_ge_2,minor_ge_1), major_ge_3 ))]
        const GDAL_OF_ARRAY_BLOCK_ACCESS = 0x100;

        /// Hashset based strategy for cached blocks.
        #[cfg(any( all(major_ge_2,minor_ge_1), major_ge_3 ))]
        const GDAL_OF_HASHSET_BLOCK_ACCESS = 0x200;
    }
}

impl Default for GdalOpenFlags {
    fn default() -> GdalOpenFlags {
        GdalOpenFlags::GDAL_OF_READONLY
    }
}

impl From<GDALAccess::Type> for GdalOpenFlags {
    fn from(val: GDALAccess::Type) -> GdalOpenFlags {
        if val == GDALAccess::GA_Update {
            GdalOpenFlags::GDAL_OF_UPDATE
        } else {
            GdalOpenFlags::GDAL_OF_READONLY
        }
    }
}

// Open parameters
#[derive(Debug, Default)]
pub struct DatasetOptions<'a> {
    pub open_flags: GdalOpenFlags,
    pub allowed_drivers: Option<&'a [&'a str]>,
    pub open_options: Option<&'a [&'a str]>,
    pub sibling_files: Option<&'a [&'a str]>,
}

/// Parameters for [`Dataset::create_layer`].
#[derive(Clone, Debug)]
pub struct LayerOptions<'a> {
    /// The name of the newly created layer. May be an empty string.
    pub name: &'a str,
    /// The SRS of the newly created layer, or `None` for no SRS.
    pub srs: Option<&'a SpatialRef>,
    /// The type of geometry for the new layer.
    pub ty: OGRwkbGeometryType::Type,
    /// Additional driver-specific options to pass to GDAL, in the form `name=value`.
    pub options: Option<&'a [&'a str]>,
}

const EMPTY_LAYER_NAME: &str = "";

impl<'a> Default for LayerOptions<'a> {
    /// Returns creation options for a new layer with no name, no SRS and unknown geometry type.
    fn default() -> Self {
        LayerOptions {
            name: EMPTY_LAYER_NAME,
            srs: None,
            ty: OGRwkbGeometryType::wkbUnknown,
            options: None,
        }
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
        let c_open_flags = options.open_flags.bits;

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
        Ok(Dataset { c_dataset })
    }

    /// Creates a new Dataset by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_dataset(c_dataset: GDALDatasetH) -> Dataset {
        Dataset { c_dataset }
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
        unsafe { SpatialRef::from_c_obj(gdal_sys::GDALGetSpatialRef(self.c_dataset)) }
    }

    #[cfg(major_ge_3)]
    /// Set the spatial reference system for this dataset.
    pub fn set_spatial_ref(&mut self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetSpatialRef(self.c_dataset, spatial_ref.to_c_hsrs()) };
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
        Self::_create_copy(self, driver, filename.as_ref(), options)
    }

    fn _create_copy(
        &self,
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
                self.c_dataset,
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

    /// Fetch the driver to which this dataset relates.
    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal_sys::GDALGetDatasetDriver(self.c_dataset);
            Driver::from_c_driver(c_driver)
        }
    }

    /// Fetch a band object for a dataset.
    ///
    /// Applies to raster datasets, and fetches the
    /// rasterband at the given _1-based_ index.
    pub fn rasterband(&self, band_index: isize) -> Result<RasterBand> {
        unsafe {
            let c_band = gdal_sys::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            if c_band.is_null() {
                return Err(_last_null_pointer_err("GDALGetRasterBand"));
            }
            Ok(RasterBand::from_c_rasterband(self, c_band))
        }
    }

    /// Builds overviews for the current `Dataset`. See [`GDALBuildOverviews`].
    ///
    /// # Arguments
    /// * `resampling` - resampling method, as accepted by GDAL, e.g. `"CUBIC"`
    /// * `overviews` - list of overview decimation factors, e.g. `&[2, 4, 8, 16, 32]`
    /// * `bands` - list of bands to build the overviews for, or empty for all bands
    ///
    /// [`GDALBuildOverviews`]: https://gdal.org/doxygen/gdal_8h.html#a767f4456a6249594ee18ea53f68b7e80
    pub fn build_overviews(
        &mut self,
        resampling: &str,
        overviews: &[i32],
        bands: &[i32],
    ) -> Result<()> {
        let c_resampling = CString::new(resampling)?;
        let rv = unsafe {
            gdal_sys::GDALBuildOverviews(
                self.c_dataset,
                c_resampling.as_ptr(),
                overviews.len() as i32,
                overviews.as_ptr() as *mut i32,
                bands.len() as i32,
                bands.as_ptr() as *mut i32,
                None,
                null_mut(),
            )
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    fn child_layer(&self, c_layer: OGRLayerH) -> Layer {
        unsafe { Layer::from_c_layer(self, c_layer) }
    }

    fn into_child_layer(self, c_layer: OGRLayerH) -> OwnedLayer {
        unsafe { OwnedLayer::from_c_layer(self, c_layer) }
    }

    /// Get the number of layers in this dataset.
    pub fn layer_count(&self) -> isize {
        (unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset) }) as isize
    }

    /// Fetch a layer by index.
    ///
    /// Applies to vector datasets, and fetches by the given
    /// _0-based_ index.
    pub fn layer(&self, idx: isize) -> Result<Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer"));
        }
        Ok(self.child_layer(c_layer))
    }

    /// Fetch a layer by index.
    ///
    /// Applies to vector datasets, and fetches by the given
    /// _0-based_ index.
    pub fn into_layer(self, idx: isize) -> Result<OwnedLayer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer"));
        }
        Ok(self.into_child_layer(c_layer))
    }

    /// Fetch a layer by name.
    pub fn layer_by_name(&self, name: &str) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName"));
        }
        Ok(self.child_layer(c_layer))
    }

    /// Fetch a layer by name.
    pub fn into_layer_by_name(self, name: &str) -> Result<OwnedLayer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName"));
        }
        Ok(self.into_child_layer(c_layer))
    }

    /// Returns an iterator over the layers of the dataset.
    pub fn layers(&self) -> LayerIterator {
        LayerIterator::with_dataset(self)
    }

    /// Fetch the number of raster bands on this dataset.
    pub fn raster_count(&self) -> isize {
        (unsafe { gdal_sys::GDALGetRasterCount(self.c_dataset) }) as isize
    }

    /// Returns the raster dimensions: (width, height).
    pub fn raster_size(&self) -> (usize, usize) {
        let size_x = unsafe { gdal_sys::GDALGetRasterXSize(self.c_dataset) } as usize;
        let size_y = unsafe { gdal_sys::GDALGetRasterYSize(self.c_dataset) } as usize;
        (size_x, size_y)
    }

    /// Creates a new layer. The [`LayerOptions`] struct implements `Default`, so you only need to
    /// specify those options that deviate from the default.
    ///
    /// # Examples
    ///
    /// Create a new layer with an empty name, no spatial reference, and unknown geometry type:
    ///
    /// ```
    /// # use gdal::Driver;
    /// # let driver = Driver::get("GPKG").unwrap();
    /// # let mut dataset = driver.create_vector_only("/vsimem/example.gpkg").unwrap();
    /// let blank_layer = dataset.create_layer(Default::default()).unwrap();
    /// ```
    ///
    /// Create a new named line string layer using WGS84:
    ///
    /// ```
    /// # use gdal::{Driver, LayerOptions};
    /// # use gdal::spatial_ref::SpatialRef;
    /// # let driver = Driver::get("GPKG").unwrap();
    /// # let mut dataset = driver.create_vector_only("/vsimem/example.gpkg").unwrap();
    /// let roads = dataset.create_layer(LayerOptions {
    ///     name: "roads",
    ///     srs: Some(&SpatialRef::from_epsg(4326).unwrap()),
    ///     ty: gdal_sys::OGRwkbGeometryType::wkbLineString,
    ///     ..Default::default()
    /// }).unwrap();
    /// ```
    pub fn create_layer<'a>(&mut self, options: LayerOptions<'a>) -> Result<Layer> {
        let c_name = CString::new(options.name)?;
        let c_srs = match options.srs {
            Some(srs) => srs.to_c_hsrs(),
            None => null_mut(),
        };

        // Handle string options: we need to keep the CStrings and the pointers around.
        let c_options = options.options.map(|d| {
            d.iter()
                .map(|&s| CString::new(s))
                .collect::<std::result::Result<Vec<CString>, NulError>>()
        });
        let c_options_vec = match c_options {
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(c_options_vec)) => c_options_vec,
            None => Vec::from([]),
        };
        let mut c_options_ptrs = c_options_vec.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        c_options_ptrs.push(ptr::null());

        let c_options_ptr = if options.options.is_some() {
            c_options_ptrs.as_ptr()
        } else {
            ptr::null()
        };

        let c_layer = unsafe {
            // The C function takes `char **papszOptions` without mention of `const`, and this is
            // propagated to the gdal_sys wrapper. The lack of `const` seems like a mistake in the
            // GDAL API, so we just do a cast here.
            gdal_sys::OGR_DS_CreateLayer(
                self.c_dataset,
                c_name.as_ptr(),
                c_srs,
                options.ty,
                c_options_ptr as *mut *mut libc::c_char,
            )
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
    /// # use gdal::{Dataset, LayerOptions};
    /// # use gdal::vector::LayerAccess;
    /// #
    /// fn create_point_grid(dataset: &mut Dataset) -> gdal::errors::Result<()> {
    ///     use gdal::vector::Geometry;
    ///
    ///     // Start the transaction.
    ///     let mut txn = dataset.start_transaction()?;
    ///
    ///     let mut layer = txn.dataset_mut().create_layer(LayerOptions {
    ///         name: "grid",
    ///         ty: gdal_sys::OGRwkbGeometryType::wkbPoint,
    ///         ..Default::default()
    ///     })?;
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

    /// Execute a SQL query against the Dataset. It is equivalent to calling
    /// [`GDALDatasetExecuteSQL`](https://gdal.org/api/raster_c_api.html#_CPPv421GDALDatasetExecuteSQL12GDALDatasetHPKc12OGRGeometryHPKc).
    /// Returns a [`sql::ResultSet`], which can be treated just as any other [`Layer`].
    ///
    /// Queries such as `ALTER TABLE`, `CREATE INDEX`, etc. have no [`sql::ResultSet`], and return
    /// `None`, which is distinct from an empty [`sql::ResultSet`].
    ///
    /// # Arguments
    ///
    /// * `query`: The SQL query
    /// * `spatial_filter`: Limit results of the query to features that intersect the given
    ///   [`Geometry`]
    /// * `dialect`: The dialect of SQL to use. See
    ///   <https://gdal.org/user/ogr_sql_sqlite_dialect.html>
    ///
    /// # Example
    ///
    /// ```
    /// # use gdal::Dataset;
    /// # use std::path::Path;
    /// use gdal::vector::sql;
    /// use gdal::vector::LayerAccess;
    ///
    /// let ds = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
    /// let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'pedestrian'";
    /// let mut result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT).unwrap().unwrap();
    ///
    /// assert_eq!(10, result_set.feature_count());
    ///
    /// for feature in result_set.features() {
    ///     let highway = feature
    ///         .field("highway")
    ///         .unwrap()
    ///         .unwrap()
    ///         .into_string()
    ///         .unwrap();
    ///
    ///     assert_eq!("pedestrian", highway);
    /// }
    /// ```
    pub fn execute_sql<S: AsRef<str>>(
        &self,
        query: S,
        spatial_filter: Option<&Geometry>,
        dialect: sql::Dialect,
    ) -> Result<Option<sql::ResultSet>> {
        let query = CString::new(query.as_ref())?;

        let dialect_c_str = match dialect {
            sql::Dialect::DEFAULT => None,
            sql::Dialect::OGR => Some(unsafe { CStr::from_bytes_with_nul_unchecked(sql::OGRSQL) }),
            sql::Dialect::SQLITE => {
                Some(unsafe { CStr::from_bytes_with_nul_unchecked(sql::SQLITE) })
            }
        };

        self._execute_sql(query, spatial_filter, dialect_c_str)
    }

    fn _execute_sql(
        &self,
        query: CString,
        spatial_filter: Option<&Geometry>,
        dialect_c_str: Option<&CStr>,
    ) -> Result<Option<sql::ResultSet>> {
        let mut filter_geom: OGRGeometryH = std::ptr::null_mut();

        let dialect_ptr = match dialect_c_str {
            None => std::ptr::null(),
            Some(d) => d.as_ptr(),
        };

        if let Some(spatial_filter) = spatial_filter {
            filter_geom = unsafe { spatial_filter.c_geometry() };
        }

        let c_dataset = unsafe { self.c_dataset() };

        unsafe { gdal_sys::CPLErrorReset() };

        let c_layer = unsafe {
            gdal_sys::GDALDatasetExecuteSQL(c_dataset, query.as_ptr(), filter_geom, dialect_ptr)
        };

        let cpl_err = unsafe { gdal_sys::CPLGetLastErrorType() };

        if cpl_err != CPLErr::CE_None {
            return Err(_last_cpl_err(cpl_err));
        }

        if c_layer.is_null() {
            return Ok(None);
        }

        let layer = unsafe { Layer::from_c_layer(self, c_layer) };

        Ok(Some(sql::ResultSet {
            layer,
            dataset: c_dataset,
        }))
    }
}

pub struct LayerIterator<'a> {
    dataset: &'a Dataset,
    idx: isize,
    count: isize,
}

impl<'a> Iterator for LayerIterator<'a> {
    type Item = Layer<'a>;

    #[inline]
    fn next(&mut self) -> Option<Layer<'a>> {
        let idx = self.idx;
        if idx < self.count {
            self.idx += 1;
            let c_layer =
                unsafe { gdal_sys::OGR_DS_GetLayer(self.dataset.c_dataset, idx as c_int) };
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
        LayerIterator {
            dataset,
            idx: 0,
            count: dataset.layer_count(),
        }
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
    use crate::vector::{Geometry, LayerAccess};
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
            DatasetOptions {
                open_flags: GDALAccess::GA_Update.into(),
                allowed_drivers: Some(&["GPKG"]),
                ..DatasetOptions::default()
            },
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
            fixture!("roads.geojson"),
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
            fixture!("roads.geojson"),
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
            fixture!("roads.geojson"),
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
            fixture!("roads.geojson"),
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
            fixture!("roads.geojson"),
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
            fixture!("roads.geojson"),
            DatasetOptions {
                open_flags: GdalOpenFlags::GDAL_OF_UPDATE | GdalOpenFlags::GDAL_OF_RASTER,
                ..DatasetOptions::default()
            },
        )
        .unwrap_err();
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
    fn test_create_layer_options() {
        use gdal_sys::OGRwkbGeometryType::wkbPoint;
        let (_temp_path, mut ds) = open_gpkg_for_update(fixture!("poly.gpkg"));
        let mut options = LayerOptions {
            name: "new",
            ty: wkbPoint,
            ..Default::default()
        };
        ds.create_layer(options.clone()).unwrap();
        assert!(ds.create_layer(options.clone()).is_err());
        options.options = Some(&["OVERWRITE=YES"]);
        assert!(ds.create_layer(options).is_ok());
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
