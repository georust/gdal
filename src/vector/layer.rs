use std::{
    ffi::{c_char, c_int, CString, NulError},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::null_mut,
};

use gdal_sys::{GDALMajorObjectH, OGRErr, OGRFieldDefnH, OGRFieldType, OGRLayerH};

use crate::errors::*;
use crate::metadata::Metadata;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::defn::Defn;
use crate::vector::feature::{FeatureIterator, OwnedFeatureIterator};
use crate::vector::{Envelope, Feature, Geometry, LayerOptions};
use crate::{dataset::Dataset, gdal_major_object::MajorObject};

/// Layer capabilities
#[allow(clippy::upper_case_acronyms)]
pub enum LayerCaps {
    /// Layer capability for random read
    OLCRandomRead,
    /// Layer capability for sequential write
    OLCSequentialWrite,
    /// Layer capability for random write
    OLCRandomWrite,
    /// Layer capability for fast spatial filter
    OLCFastSpatialFilter,
    /// Layer capability for fast feature count retrieval
    OLCFastFeatureCount,
    /// Layer capability for fast extent retrieval
    OLCFastGetExtent,
    /// Layer capability for field creation
    OLCCreateField,
    /// Layer capability for field deletion
    OLCDeleteField,
    /// Layer capability for field reordering
    OLCReorderFields,
    /// Layer capability for field alteration
    OLCAlterFieldDefn,
    /// Layer capability for transactions
    OLCTransactions,
    /// Layer capability for feature deletion
    OLCDeleteFeature,
    /// Layer capability for setting next feature index
    OLCFastSetNextByIndex,
    /// Layer capability for strings returned with UTF-8 encoding
    OLCStringsAsUTF8,
    /// Layer capability for field ignoring
    OLCIgnoreFields,
    /// Layer capability for geometry field creation
    OLCCreateGeomField,
    /// Layer capability for curve geometries support
    OLCCurveGeometries,
    /// Layer capability for measured geometries support
    OLCMeasuredGeometries,
    /// Layer capability for a specialized implementation to ArrowArrayStream
    OLCFastGetArrowStream,
}

// Manage conversion to Gdal values
impl LayerCaps {
    fn into_cstring(self) -> CString {
        CString::new(match self {
            Self::OLCRandomRead => "RandomRead",
            Self::OLCSequentialWrite => "SequentialWrite",
            Self::OLCRandomWrite => "RandomWrite",
            Self::OLCFastSpatialFilter => "FastSpatialFilter",
            Self::OLCFastFeatureCount => "FastFeatureCount",
            Self::OLCFastGetExtent => "FastGetExtent",
            Self::OLCCreateField => "CreateField",
            Self::OLCDeleteField => "DeleteField",
            Self::OLCReorderFields => "ReorderFields",
            Self::OLCAlterFieldDefn => "AlterFieldDefn",
            Self::OLCTransactions => "Transactions",
            Self::OLCDeleteFeature => "DeleteFeature",
            Self::OLCFastSetNextByIndex => "FastSetNextByIndex",
            Self::OLCStringsAsUTF8 => "StringsAsUTF8",
            Self::OLCIgnoreFields => "IgnoreFields",
            Self::OLCCreateGeomField => "CreateGeomField",
            Self::OLCCurveGeometries => "CurveGeometries",
            Self::OLCMeasuredGeometries => "MeasuredGeometries",
            Self::OLCFastGetArrowStream => "FastGetArrowStream",
        })
        .unwrap()
    }
}

/// Layer in a vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::Dataset;
/// use gdal::vector::LayerAccess;
///
/// let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// let mut layer = dataset.layer(0).unwrap();
/// for feature in layer.features() {
///     // do something with each feature
/// }
/// ```
#[derive(Debug)]
pub struct Layer<'a> {
    c_layer: OGRLayerH,
    defn: Defn,
    phantom: PhantomData<&'a Dataset>,
}

impl MajorObject for Layer<'_> {
    fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_layer
    }
}

impl Metadata for Layer<'_> {}

impl LayerAccess for Layer<'_> {
    unsafe fn c_layer(&self) -> OGRLayerH {
        self.c_layer
    }

    fn defn(&self) -> &Defn {
        &self.defn
    }
}

impl<'a> Layer<'a> {
    /// Creates a new Layer from a GDAL layer pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub(crate) unsafe fn from_c_layer(_: &'a Dataset, c_layer: OGRLayerH) -> Self {
        let c_defn = gdal_sys::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::from_c_defn(c_defn);
        Self {
            c_layer,
            defn,
            phantom: PhantomData,
        }
    }
}

/// Layer in a vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::Dataset;
/// use gdal::vector::LayerAccess;
///
/// let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// let mut layer = dataset.into_layer(0).unwrap();
/// for feature in layer.features() {
///     // do something with each feature
/// }
/// ```
#[derive(Debug)]
pub struct OwnedLayer {
    c_layer: OGRLayerH,
    defn: Defn,
    // we store the dataset to prevent dropping (i.e. closing) it
    _dataset: Dataset,
}

impl MajorObject for OwnedLayer {
    fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_layer
    }
}

impl Metadata for OwnedLayer {}

impl LayerAccess for OwnedLayer {
    unsafe fn c_layer(&self) -> OGRLayerH {
        self.c_layer
    }

    fn defn(&self) -> &Defn {
        &self.defn
    }
}

impl OwnedLayer {
    /// Creates a new Layer from a GDAL layer pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub(crate) unsafe fn from_c_layer(dataset: Dataset, c_layer: OGRLayerH) -> Self {
        let c_defn = gdal_sys::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::from_c_defn(c_defn);
        Self {
            c_layer,
            defn,
            _dataset: dataset,
        }
    }

    /// Returns iterator over the features in this layer.
    ///
    /// **Note.** This method resets the current index to
    /// the beginning before iteration. It also borrows the
    /// layer mutably, preventing any overlapping borrows.
    pub fn owned_features(mut self) -> OwnedFeatureIterator {
        self.reset_feature_reading();
        OwnedFeatureIterator::_with_layer(self)
    }

    /// Returns the `Dataset` this layer belongs to and consumes this layer.
    pub fn into_dataset(self) -> Dataset {
        self._dataset
    }
}

/// As long we have a 1:1 mapping between a dataset and a layer, it is `Send`.
/// We cannot allow a layer to be send, when two or more access (and modify) the same `Dataset`.
unsafe impl Send for OwnedLayer {}

impl From<OwnedLayer> for Dataset {
    fn from(owned_layer: OwnedLayer) -> Self {
        owned_layer.into_dataset()
    }
}

pub trait LayerAccess: Sized {
    /// Returns the C wrapped pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    unsafe fn c_layer(&self) -> OGRLayerH;

    fn defn(&self) -> &Defn;

    /// Returns the feature with the given feature id `fid`, or `None` if not found.
    ///
    /// This function is unaffected by the spatial or attribute filters.
    ///
    /// Not all drivers support this efficiently; however, the call should always work if the
    /// feature exists, as a fallback implementation just scans all the features in the layer
    /// looking for the desired feature.
    fn feature(&self, fid: u64) -> Option<Feature> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetFeature(self.c_layer(), fid as i64) };
        if c_feature.is_null() {
            None
        } else {
            Some(unsafe { Feature::from_c_feature(self.defn(), c_feature) })
        }
    }

    /// Returns iterator over the features in this layer.
    ///
    /// **Note.** This method resets the current index to
    /// the beginning before iteration. It also borrows the
    /// layer mutably, preventing any overlapping borrows.
    fn features(&mut self) -> FeatureIterator {
        self.reset_feature_reading();
        FeatureIterator::_with_layer(self)
    }

    /// Set a feature on this layer layer.
    ///
    /// See: [SetFeature](https://gdal.org/doxygen/classOGRLayer.html#a681139bfd585b74d7218e51a32144283)
    fn set_feature(&self, feature: Feature) -> Result<()> {
        unsafe { gdal_sys::OGR_L_SetFeature(self.c_layer(), feature.c_feature()) };
        Ok(())
    }

    /// Set a spatial filter on this layer.
    ///
    /// See: [OGR_L_SetSpatialFilter](https://gdal.org/doxygen/classOGRLayer.html#a75c06b4993f8eb76b569f37365cd19ab)
    fn set_spatial_filter(&mut self, geometry: &Geometry) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer(), geometry.c_geometry()) };
    }

    /// Set a spatial rectangle filter on this layer by specifying the bounds of a rectangle.
    fn set_spatial_filter_rect(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilterRect(self.c_layer(), min_x, min_y, max_x, max_y) };
    }

    /// Clear spatial filters set on this layer.
    fn clear_spatial_filter(&mut self) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer(), null_mut()) };
    }

    /// Get the name of this layer.
    fn name(&self) -> String {
        let rv = unsafe { gdal_sys::OGR_L_GetName(self.c_layer()) };
        _string(rv).unwrap_or_default()
    }

    fn has_capability(&self, capability: LayerCaps) -> bool {
        unsafe {
            gdal_sys::OGR_L_TestCapability(self.c_layer(), capability.into_cstring().as_ptr()) == 1
        }
    }

    fn create_defn_fields(&self, fields_def: &[(&str, OGRFieldType::Type)]) -> Result<()> {
        for fd in fields_def {
            let fdefn = FieldDefn::new(fd.0, fd.1)?;
            fdefn.add_to_layer(self)?;
        }
        Ok(())
    }
    fn create_feature(&mut self, geometry: Geometry) -> Result<()> {
        let feature = Feature::new(self.defn())?;

        let c_geometry = unsafe { geometry.into_c_geometry() };
        let rv = unsafe { gdal_sys::OGR_F_SetGeometryDirectly(feature.c_feature(), c_geometry) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_F_SetGeometryDirectly",
            });
        }
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(self.c_layer(), feature.c_feature()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            });
        }
        Ok(())
    }

    /// Returns the number of features in this layer, even if it requires expensive calculation.
    ///
    /// Some drivers will actually scan the entire layer once to count objects.
    ///
    /// The returned count takes the [spatial filter](`Layer::set_spatial_filter`) into account.
    /// For dynamic databases the count may not be exact.
    fn feature_count(&self) -> u64 {
        (unsafe { gdal_sys::OGR_L_GetFeatureCount(self.c_layer(), 1) }) as u64
    }

    /// Returns the number of features in this layer, if it is possible to compute this
    /// efficiently.
    ///
    /// For some drivers, it would be expensive to establish the feature count, in which case
    /// [`None`] will be returned.
    ///
    /// The returned count takes the [spatial filter](`Layer::set_spatial_filter`) into account.
    /// For dynamic databases the count may not be exact.
    fn try_feature_count(&self) -> Option<u64> {
        let rv = unsafe { gdal_sys::OGR_L_GetFeatureCount(self.c_layer(), 0) };
        if rv < 0 {
            None
        } else {
            Some(rv as u64)
        }
    }

    /// Returns the extent of this layer as an axis-aligned bounding box, even if it requires
    /// expensive calculation.
    ///
    /// Some drivers will actually scan the entire layer once to count objects.
    ///
    /// Depending on the driver, the returned extent may or may not take the [spatial
    /// filter](`Layer::set_spatial_filter`) into account. So it is safer to call `get_extent`
    /// without setting a spatial filter.
    ///
    /// Layers without any geometry may return [`OGRErr::OGRERR_FAILURE`] to indicate that no
    /// meaningful extents could be collected.
    fn get_extent(&self) -> Result<Envelope> {
        let mut envelope = MaybeUninit::uninit();
        let force = 1;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer(), envelope.as_mut_ptr(), force) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_GetExtent",
            });
        }
        Ok(unsafe { envelope.assume_init() })
    }

    /// Returns the extent of this layer as an axis-aligned bounding box, if it is possible to
    /// compute this efficiently.
    ///
    /// For some drivers, it would be expensive to calculate the extent, in which case [`None`]
    /// will be returned.
    ///
    /// Depending on the driver, the returned extent may or may not take the [spatial
    /// filter](`Layer::set_spatial_filter`) into account. So it is safer to call `try_get_extent`
    /// without setting a spatial filter.
    fn try_get_extent(&self) -> Result<Option<Envelope>> {
        let mut envelope = MaybeUninit::uninit();
        let force = 0;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer(), envelope.as_mut_ptr(), force) };
        if rv == OGRErr::OGRERR_FAILURE {
            Ok(None)
        } else {
            if rv != OGRErr::OGRERR_NONE {
                return Err(GdalError::OgrError {
                    err: rv,
                    method_name: "OGR_L_GetExtent",
                });
            }
            Ok(Some(unsafe { envelope.assume_init() }))
        }
    }

    /// Get the spatial reference system for this layer.
    ///
    /// Returns `Some(SpatialRef)`, or `None` if one isn't defined.
    ///
    /// See: [OGR_L_GetSpatialRef](https://gdal.org/doxygen/classOGRLayer.html#a75c06b4993f8eb76b569f37365cd19ab)
    fn spatial_ref(&self) -> Option<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OGR_L_GetSpatialRef(self.c_layer()) };
        if c_obj.is_null() {
            None
        } else {
            unsafe { SpatialRef::from_c_obj(c_obj) }.ok()
        }
    }

    fn reset_feature_reading(&mut self) {
        unsafe {
            gdal_sys::OGR_L_ResetReading(self.c_layer());
        }
    }

    /// Set a new attribute query that restricts features when using the feature iterator.
    ///
    /// From the GDAL docs: Note that installing a query string will generally result in resetting the current reading position
    ///
    /// Parameters:
    /// - `query` in restricted SQL WHERE format
    ///
    fn set_attribute_filter(&mut self, query: &str) -> Result<()> {
        let c_str = CString::new(query)?;
        let rv = unsafe { gdal_sys::OGR_L_SetAttributeFilter(self.c_layer(), c_str.as_ptr()) };

        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_SetAttributeFilter",
            });
        }

        Ok(())
    }

    /// Clear the attribute filter set on this layer
    ///
    /// From the GDAL docs: Note that installing a query string will generally result in resetting the current reading position
    ///
    fn clear_attribute_filter(&mut self) {
        unsafe {
            gdal_sys::OGR_L_SetAttributeFilter(self.c_layer(), null_mut());
        }
    }

    /// Read batches of columnar [Arrow](https://arrow.apache.org/) data from OGR.
    ///
    /// Extended options are available via [`crate::cpl::CslStringList`].
    /// As defined in the OGR documentation for [`GetArrowStream`](https://gdal.org/api/ogrlayer_cpp.html#_CPPv4N8OGRLayer14GetArrowStreamEP16ArrowArrayStream12CSLConstList),
    /// the current options are:
    ///
    /// * `INCLUDE_FID=YES/NO`. Whether to include the FID column. Defaults to YES.
    /// * `MAX_FEATURES_IN_BATCH=integer`. Maximum number of features to retrieve in a ArrowArray batch. Defaults to 65 536.
    ///
    /// Additional driver-specific options may exist.
    ///
    /// This API is new as of GDAL 3.6.
    ///
    /// # Example
    ///
    /// Refer to the example provided in `read_ogr_arrow.rs`.
    ///
    /// # Safety
    /// This uses the Arrow C Data Interface to operate on raw pointers provisioned from Rust.
    /// These pointers must be valid and provisioned according to the ArrowArrayStream spec.
    #[cfg(any(major_ge_4, all(major_is_3, minor_ge_6)))]
    unsafe fn read_arrow_stream(
        &mut self,
        out_stream: *mut gdal_sys::ArrowArrayStream,
        options: &crate::cpl::CslStringList,
    ) -> Result<()> {
        self.reset_feature_reading();

        unsafe {
            let success =
                gdal_sys::OGR_L_GetArrowStream(self.c_layer(), out_stream, options.as_ptr());
            if !success {
                return Err(GdalError::OgrError {
                    err: 1,
                    method_name: "OGR_L_GetArrowStream",
                });
            }
        }

        Ok(())
    }
}

pub struct LayerIterator<'a> {
    dataset: &'a Dataset,
    idx: usize,
    count: usize,
}

impl<'a> Iterator for LayerIterator<'a> {
    type Item = Layer<'a>;

    #[inline]
    fn next(&mut self) -> Option<Layer<'a>> {
        let idx = self.idx;
        if idx < self.count {
            self.idx += 1;
            let c_layer =
                unsafe { gdal_sys::GDALDatasetGetLayer(self.dataset.c_dataset(), idx as c_int) };
            if !c_layer.is_null() {
                let layer = unsafe { Layer::from_c_layer(self.dataset, c_layer) };
                return Some(layer);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.count;
        (size, Some(size))
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
pub struct FieldDefn {
    c_obj: OGRFieldDefnH,
}

impl Drop for FieldDefn {
    fn drop(&mut self) {
        unsafe { gdal_sys::OGR_Fld_Destroy(self.c_obj) };
    }
}

impl MajorObject for FieldDefn {
    fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_obj
    }
}

impl FieldDefn {
    pub fn new(name: &str, field_type: OGRFieldType::Type) -> Result<FieldDefn> {
        let c_str = CString::new(name)?;
        let c_obj = unsafe { gdal_sys::OGR_Fld_Create(c_str.as_ptr(), field_type) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_Fld_Create"));
        };
        Ok(FieldDefn { c_obj })
    }
    pub fn set_width(&self, width: i32) {
        unsafe { gdal_sys::OGR_Fld_SetWidth(self.c_obj, width as c_int) };
    }
    pub fn set_precision(&self, precision: i32) {
        unsafe { gdal_sys::OGR_Fld_SetPrecision(self.c_obj, precision as c_int) };
    }
    pub fn add_to_layer<L: LayerAccess>(&self, layer: &L) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_L_CreateField(layer.c_layer(), self.c_obj, 1) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            });
        }
        Ok(())
    }
}

/// [Layer] related methods for [Dataset].
impl Dataset {
    fn child_layer(&self, c_layer: OGRLayerH) -> Layer {
        unsafe { Layer::from_c_layer(self, c_layer) }
    }

    fn into_child_layer(self, c_layer: OGRLayerH) -> OwnedLayer {
        unsafe { OwnedLayer::from_c_layer(self, c_layer) }
    }

    /// Get the number of layers in this dataset.
    pub fn layer_count(&self) -> usize {
        (unsafe { gdal_sys::GDALDatasetGetLayerCount(self.c_dataset()) }) as usize
    }

    /// Fetch a layer by index.
    ///
    /// Applies to vector datasets, and fetches by the given
    /// _0-based_ index.
    pub fn layer(&self, idx: usize) -> Result<Layer> {
        let idx = c_int::try_from(idx)?;
        let c_layer = unsafe { gdal_sys::GDALDatasetGetLayer(self.c_dataset(), idx) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("GDALDatasetGetLayer"));
        }
        Ok(self.child_layer(c_layer))
    }

    /// Fetch a layer by index.
    ///
    /// Applies to vector datasets, and fetches by the given
    /// _0-based_ index.
    pub fn into_layer(self, idx: usize) -> Result<OwnedLayer> {
        let idx = c_int::try_from(idx)?;
        let c_layer = unsafe { gdal_sys::GDALDatasetGetLayer(self.c_dataset(), idx) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("GDALDatasetGetLayer"));
        }
        Ok(self.into_child_layer(c_layer))
    }

    /// Fetch a layer by name.
    pub fn layer_by_name(&self, name: &str) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_layer =
            unsafe { gdal_sys::GDALDatasetGetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("GDALDatasetGetLayerByName"));
        }
        Ok(self.child_layer(c_layer))
    }

    /// Fetch a layer by name.
    pub fn into_layer_by_name(self, name: &str) -> Result<OwnedLayer> {
        let c_name = CString::new(name)?;
        let c_layer =
            unsafe { gdal_sys::GDALDatasetGetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("GDALDatasetGetLayerByName"));
        }
        Ok(self.into_child_layer(c_layer))
    }

    /// Returns an iterator over the layers of the dataset.
    pub fn layers(&self) -> LayerIterator {
        LayerIterator::with_dataset(self)
    }

    /// Creates a new layer. The [`LayerOptions`] struct implements `Default`, so you only need to
    /// specify those options that deviate from the default.
    ///
    /// # Examples
    ///
    /// Create a new layer with an empty name, no spatial reference, and unknown geometry type:
    ///
    /// ```
    /// # use gdal::DriverManager;
    /// # let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    /// # let mut dataset = driver.create_vector_only("/vsimem/example.gpkg").unwrap();
    /// let blank_layer = dataset.create_layer(Default::default()).unwrap();
    /// ```
    ///
    /// Create a new named line string layer using WGS84:
    ///
    /// ```
    /// # use gdal::{DriverManager };
    /// # use gdal::spatial_ref::SpatialRef;
    /// # use gdal::vector::LayerOptions;
    /// # let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    /// # let mut dataset = driver.create_vector_only("/vsimem/example.gpkg").unwrap();
    /// let roads = dataset.create_layer(LayerOptions {
    ///     name: "roads",
    ///     srs: Some(&SpatialRef::from_epsg(4326).unwrap()),
    ///     ty: gdal_sys::OGRwkbGeometryType::wkbLineString,
    ///     ..Default::default()
    /// }).unwrap();
    /// ```
    pub fn create_layer(&mut self, options: LayerOptions<'_>) -> Result<Layer> {
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
        c_options_ptrs.push(std::ptr::null());

        let c_options_ptr = if options.options.is_some() {
            c_options_ptrs.as_ptr()
        } else {
            std::ptr::null()
        };

        let c_layer = unsafe {
            // The C function takes `char **papszOptions` without mention of `const`, and this is
            // propagated to the gdal_sys wrapper. The lack of `const` seems like a mistake in the
            // GDAL API, so we just do a cast here.
            gdal_sys::GDALDatasetCreateLayer(
                self.c_dataset(),
                c_name.as_ptr(),
                c_srs,
                options.ty,
                c_options_ptr as *mut *mut c_char,
            )
        };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("GDALDatasetCreateLayer"));
        };
        Ok(self.child_layer(c_layer))
    }

    /// Deletes the layer at given index
    ///
    /// ```
    /// # use gdal::DriverManager;
    /// # let driver = DriverManager::get_driver_by_name("GPKG").unwrap();
    /// # let mut dataset = driver.create_vector_only("/vsimem/example.gpkg").unwrap();
    /// let blank_layer = dataset.create_layer(Default::default()).unwrap();
    /// assert!(dataset.delete_layer(1).is_err());
    /// dataset.delete_layer(0).unwrap();
    /// assert_eq!(dataset.layers().count(), 0);
    /// ```
    pub fn delete_layer(&mut self, idx: usize) -> Result<()> {
        let idx = c_int::try_from(idx)?;
        let err = unsafe { gdal_sys::GDALDatasetDeleteLayer(self.c_dataset(), idx) };
        if err != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err,
                method_name: "GDALDatasetDeleteLayer",
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LayerCaps::*, *};
    use crate::options::DatasetOptions;
    use crate::spatial_ref::AxisMappingStrategy;
    use crate::test_utils::{fixture, open_gpkg_for_update, SuppressGDALErrorLog, TempFixture};
    use crate::vector::feature::FeatureIterator;
    use crate::vector::geometry::CoordinateLayout;
    use crate::vector::FieldValue;
    use crate::{assert_almost_eq, Dataset, DriverManager, GdalOpenFlags};
    use gdal_sys::OGRwkbGeometryType;

    fn ds_with_layer<F>(ds_name: &str, layer_name: &str, f: F)
    where
        F: Fn(Layer),
    {
        let ds = Dataset::open(fixture(ds_name)).unwrap();
        let layer = ds.layer_by_name(layer_name).unwrap();
        f(layer);
    }

    fn with_layer<F>(name: &str, f: F)
    where
        F: Fn(Layer),
    {
        let ds = Dataset::open(fixture(name)).unwrap();
        let layer = ds.layer(0).unwrap();
        f(layer);
    }

    fn with_owned_layer<F>(name: &str, f: F)
    where
        F: Fn(OwnedLayer),
    {
        let ds = Dataset::open(fixture(name)).unwrap();
        let layer = ds.into_layer(0).unwrap();
        f(layer);
    }

    fn with_features<F>(name: &str, f: F)
    where
        F: Fn(FeatureIterator),
    {
        with_layer(name, |mut layer| f(layer.features()));
    }

    fn with_feature<F>(name: &str, fid: u64, f: F)
    where
        F: Fn(Feature),
    {
        with_layer(name, |layer| f(layer.feature(fid).unwrap()));
    }

    #[test]
    fn test_create_layer_options() {
        use gdal_sys::OGRwkbGeometryType::wkbPoint;
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
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
    fn test_layer_count() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        assert_eq!(ds.layer_count(), 1);
    }

    #[test]
    fn test_many_layer_count() {
        let ds = Dataset::open(fixture("three_layer_ds.s3db")).unwrap();
        assert_eq!(ds.layer_count(), 3);
    }

    #[test]
    fn test_layer_get_extent() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        let extent = layer.get_extent().unwrap();
        assert_almost_eq(extent.MinX, 26.100768);
        assert_almost_eq(extent.MaxX, 26.103515);
        assert_almost_eq(extent.MinY, 44.429858);
        assert_almost_eq(extent.MaxY, 44.431818);
    }

    #[test]
    fn test_layer_try_get_extent() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        if cfg!(any(major_ge_4, all(major_is_3, minor_ge_9))) {
            assert!(layer.try_get_extent().unwrap().is_some());
        } else {
            assert!(layer.try_get_extent().unwrap().is_none());
        }
    }

    #[test]
    fn test_layer_spatial_ref() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        let srs = layer.spatial_ref().unwrap();
        assert_eq!(srs.auth_code().unwrap(), 4326);
    }

    #[test]
    fn test_layer_capabilities() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();

        assert!(!layer.has_capability(OLCFastSpatialFilter));
        assert!(layer.has_capability(OLCFastFeatureCount));
        if cfg!(any(major_ge_4, all(major_is_3, minor_ge_9))) {
            assert!(layer.has_capability(OLCFastGetExtent));
        } else {
            assert!(!layer.has_capability(OLCFastGetExtent));
        }
        assert!(layer.has_capability(OLCRandomRead));
        assert!(layer.has_capability(OLCStringsAsUTF8));
    }

    #[test]
    fn test_feature_count() {
        with_layer("roads.geojson", |layer| {
            assert_eq!(layer.feature_count(), 21);
        });
    }

    #[test]
    fn test_many_feature_count() {
        ds_with_layer("three_layer_ds.s3db", "layer_0", |layer| {
            assert_eq!(layer.feature_count(), 3);
        });
    }

    #[test]
    fn test_try_feature_count() {
        with_layer("roads.geojson", |layer| {
            assert_eq!(layer.try_feature_count(), Some(21));
        });
    }

    #[test]
    fn test_feature() {
        with_layer("roads.geojson", |layer| {
            assert!(layer.feature(236194095).is_some());
            assert!(layer.feature(23489660).is_some());
            assert!(layer.feature(0).is_none());
            assert!(layer.feature(404).is_none());
        });
    }

    #[test]
    fn test_iterate_features() {
        with_features("roads.geojson", |features| {
            assert_eq!(features.size_hint(), (21, Some(21)));
            assert_eq!(features.count(), 21);
        });
    }

    #[test]
    fn test_iterate_layers() {
        let ds = Dataset::open(fixture("three_layer_ds.s3db")).unwrap();
        let layers = ds.layers();
        assert_eq!(layers.size_hint(), (3, Some(3)));
        assert_eq!(layers.count(), 3);
    }

    #[test]
    fn test_owned_layers() {
        let ds = Dataset::open(fixture("three_layer_ds.s3db")).unwrap();

        assert_eq!(ds.layer_count(), 3);

        let mut layer = ds.into_layer(0).unwrap();

        {
            let feature = layer.features().next().unwrap();
            let id_idx = feature.field_index("id").unwrap();
            assert_eq!(feature.field(id_idx).unwrap(), None);
        }

        // convert back to dataset

        let ds = layer.into_dataset();
        assert_eq!(ds.layer_count(), 3);
    }

    #[test]
    fn test_iterate_owned_features() {
        with_owned_layer("roads.geojson", |layer| {
            let mut features = layer.owned_features();

            assert_eq!(features.as_mut().size_hint(), (21, Some(21)));
            assert_eq!(features.count(), 21);

            // get back layer

            let layer = features.into_layer();
            assert_eq!(layer.name(), "roads");
        });
    }

    #[test]
    fn test_fid() {
        with_feature("roads.geojson", 236194095, |feature| {
            assert_eq!(feature.fid(), Some(236194095));
        });
    }

    #[test]
    fn test_string_field() {
        with_feature("roads.geojson", 236194095, |feature| {
            let highway_idx = feature.field_index("highway").unwrap();
            assert_eq!(
                feature.field(highway_idx).unwrap().unwrap().into_string(),
                Some("footway".to_string())
            );
        });
        with_features("roads.geojson", |features| {
            assert_eq!(
                features
                    .filter(|feature| {
                        let highway_idx = feature.field_index("highway").unwrap();
                        let highway = feature.field(highway_idx).unwrap().unwrap().into_string();
                        highway == Some("residential".to_string())
                    })
                    .count(),
                2
            );
        });
    }

    #[test]
    fn test_null_field() {
        with_features("null_feature_fields.geojson", |mut features| {
            let feature = features.next().unwrap();
            let some_int_idx = feature.field_index("some_int").unwrap();
            let some_string_idx = feature.field_index("some_string").unwrap();
            assert_eq!(
                feature.field(some_int_idx).unwrap(),
                Some(FieldValue::IntegerValue(0))
            );
            assert_eq!(feature.field(some_string_idx).unwrap(), None);
        });
    }

    #[test]
    fn test_string_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let a_string_list_idx = feature.field_index("a_string_list").unwrap();
            assert_eq!(
                feature.field(a_string_list_idx).unwrap().unwrap(),
                FieldValue::StringListValue(vec![
                    String::from("a"),
                    String::from("list"),
                    String::from("of"),
                    String::from("strings")
                ])
            );
        });
    }

    #[test]
    fn test_set_string_list_field() {
        with_features("soundg.json", |mut features| {
            let mut feature = features.next().unwrap();
            let a_string_list_idx = feature.field_index("a_string_list").unwrap();
            let value = FieldValue::StringListValue(vec![
                String::from("the"),
                String::from("new"),
                String::from("strings"),
            ]);
            feature.set_field(a_string_list_idx, &value).unwrap();
            assert_eq!(feature.field(a_string_list_idx).unwrap().unwrap(), value);
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_get_field_as_x_by_name() {
        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();

            let sort_key_idx = feature.field_index("sort_key").unwrap();
            let highway_idx = feature.field_index("highway").unwrap();
            let railway_idx = feature.field_index("railway").unwrap();

            assert_eq!(
                feature.field_as_string(highway_idx).unwrap(),
                Some("footway".to_owned())
            );

            assert_eq!(
                feature.field_as_string(sort_key_idx).unwrap(),
                Some("-9".to_owned())
            );
            assert_eq!(feature.field_as_integer(sort_key_idx).unwrap(), Some(-9));
            assert_eq!(feature.field_as_integer64(sort_key_idx).unwrap(), Some(-9));
            assert_eq!(feature.field_as_double(sort_key_idx).unwrap(), Some(-9.));

            // test failed conversions
            assert_eq!(feature.field_as_integer(highway_idx).unwrap(), Some(0));
            assert_eq!(feature.field_as_integer64(highway_idx).unwrap(), Some(0));
            assert_eq!(feature.field_as_double(highway_idx).unwrap(), Some(0.));

            // test nulls
            assert_eq!(feature.field_as_string(railway_idx).unwrap(), None);
            assert_eq!(feature.field_as_integer(railway_idx).unwrap(), None);
            assert_eq!(feature.field_as_integer64(railway_idx).unwrap(), None);
            assert_eq!(feature.field_as_double(railway_idx).unwrap(), None);

            assert!(matches!(
                feature.field_index("not_a_field").unwrap_err(),
                GdalError::InvalidFieldName {
                    field_name,
                    method_name: "OGR_F_GetFieldIndex",
                }
                if field_name == "not_a_field"
            ));
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_get_field_as_x() {
        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();

            let highway_field = 6;
            let railway_field = 5;
            let sort_key_field = 1;

            assert_eq!(
                feature.field_as_string(highway_field).unwrap(),
                Some("footway".to_owned())
            );

            assert_eq!(
                feature.field_as_string(sort_key_field).unwrap(),
                Some("-9".to_owned())
            );
            assert_eq!(feature.field_as_integer(sort_key_field).unwrap(), Some(-9));
            assert_eq!(
                feature.field_as_integer64(sort_key_field).unwrap(),
                Some(-9)
            );
            assert_eq!(feature.field_as_double(sort_key_field).unwrap(), Some(-9.));

            // test failed conversions
            assert_eq!(feature.field_as_integer(highway_field).unwrap(), Some(0));
            assert_eq!(feature.field_as_integer64(highway_field).unwrap(), Some(0));
            assert_eq!(feature.field_as_double(highway_field).unwrap(), Some(0.));

            // test nulls
            assert_eq!(feature.field_as_string(railway_field).unwrap(), None);
            assert_eq!(feature.field_as_integer(railway_field).unwrap(), None);
            assert_eq!(feature.field_as_integer64(railway_field).unwrap(), None);
            assert_eq!(feature.field_as_double(railway_field).unwrap(), None);

            // test error
            assert!(matches!(
                feature.field_as_string(23).unwrap_err(),
                GdalError::InvalidFieldIndex {
                    index: 23,
                    method_name: "field_as_string",
                }
            ));
        });
    }

    #[test]
    fn test_get_field_as_datetime() {
        use chrono::{FixedOffset, TimeZone};

        let hour_secs = 3600;

        with_features("points_with_datetime.json", |mut features| {
            let feature = features.next().unwrap();

            let dt = FixedOffset::east_opt(-5 * hour_secs)
                .unwrap()
                .with_ymd_and_hms(2011, 7, 14, 19, 43, 37)
                .unwrap();

            let d = FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2018, 1, 4, 0, 0, 0)
                .unwrap();

            let dt_idx = feature.field_index("dt").unwrap();
            let d_idx = feature.field_index("d").unwrap();

            assert_eq!(feature.field_as_datetime(dt_idx).unwrap(), Some(dt));
            assert_eq!(feature.field_as_datetime(d_idx).unwrap(), Some(d));
        });

        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();

            let railway_field = 5;

            // test null
            assert_eq!(feature.field_as_datetime(railway_field).unwrap(), None);
            assert_eq!(feature.field_as_datetime(railway_field).unwrap(), None);

            // test error
            assert!(matches!(
                feature.field_index("not_a_field")
                    .unwrap_err(),
                GdalError::InvalidFieldName {
                    field_name,
                    method_name: "OGR_F_GetFieldIndex",
                } if field_name == "not_a_field"
            ));
            assert!(matches!(
                feature.field_as_datetime(23).unwrap_err(),
                GdalError::InvalidFieldIndex {
                    index: 23,
                    method_name: "field_as_datetime",
                }
            ));
        });
    }

    #[test]
    fn test_field_in_layer() {
        ds_with_layer("three_layer_ds.s3db", "layer_0", |mut layer| {
            let feature = layer.features().next().unwrap();
            let id_idx = feature.field_index("id").unwrap();
            assert_eq!(feature.field(id_idx).unwrap(), None);
        });
    }

    #[test]
    fn test_int_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let an_int_list_idx = feature.field_index("an_int_list").unwrap();
            assert_eq!(
                feature.field(an_int_list_idx).unwrap().unwrap(),
                FieldValue::IntegerListValue(vec![1, 2])
            );
        });
    }

    #[test]
    fn test_set_int_list_field() {
        with_features("soundg.json", |mut features| {
            let mut feature = features.next().unwrap();
            let value = FieldValue::IntegerListValue(vec![3, 4, 5]);
            let an_int_list_idx = feature.field_index("an_int_list").unwrap();
            feature.set_field(an_int_list_idx, &value).unwrap();
            assert_eq!(feature.field(an_int_list_idx).unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_real_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let a_real_list_idx = feature.field_index("a_real_list").unwrap();
            assert_eq!(
                feature.field(a_real_list_idx).unwrap().unwrap(),
                FieldValue::RealListValue(vec![0.1, 0.2])
            );
        });
    }

    #[test]
    fn test_set_real_list_field() {
        with_features("soundg.json", |mut features| {
            let mut feature = features.next().unwrap();
            let a_real_list_idx = feature.field_index("a_real_list").unwrap();
            let value = FieldValue::RealListValue(vec![2.5, 3.0, 4.75]);
            feature.set_field(a_real_list_idx, &value).unwrap();
            assert_eq!(feature.field(a_real_list_idx).unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_long_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let a_long_list_idx = feature.field_index("a_long_list").unwrap();
            assert_eq!(
                feature.field(a_long_list_idx).unwrap().unwrap(),
                FieldValue::Integer64ListValue(vec![5000000000, 6000000000])
            );
        });
    }

    #[test]
    fn test_set_long_list_field() {
        with_features("soundg.json", |mut features| {
            let mut feature = features.next().unwrap();
            let a_long_list_idx = feature.field_index("a_long_list").unwrap();
            let value = FieldValue::Integer64ListValue(vec![7000000000, 8000000000]);
            feature.set_field(a_long_list_idx, &value).unwrap();
            assert_eq!(feature.field(a_long_list_idx).unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_float_field() {
        with_feature("roads.geojson", 236194095, |feature| {
            let sort_key_idx = feature.field_index("sort_key").unwrap();
            assert_almost_eq(
                feature
                    .field(sort_key_idx)
                    .unwrap()
                    .unwrap()
                    .into_real()
                    .unwrap(),
                -9.0,
            );
        });
    }

    #[test]
    fn test_geom_accessors() {
        with_feature("roads.geojson", 236194095, |feature| {
            let geom = feature.geometry().unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            let mut coords: Vec<f64> = Vec::new();
            geom.get_points(&mut coords, CoordinateLayout::XyXy);
            assert_eq!(
                coords,
                [26.1019276, 44.4302748, 26.1019382, 44.4303191, 26.1020002, 44.4304202]
            );
            assert_eq!(geom.geometry_count(), 0);

            let geom = feature.geometry_by_index(0).unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            assert!(feature.geometry_by_index(1).is_err());
            let geom = feature.geometry_by_index(0);
            assert!(geom.is_ok());
            let geom = feature.geometry_by_index(0).unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            assert!(feature.geometry_by_index(1).is_err());

            assert_eq!(feature.geometry_field_index("").unwrap(), 0);
            assert!(feature.geometry_field_index("FOO").is_err());
        });
    }

    #[test]
    fn test_feature_wkt() {
        with_feature("roads.geojson", 236194095, |feature| {
            let wkt = feature.geometry().unwrap().wkt().unwrap();
            let wkt_ok = format!(
                "{}{}",
                "LINESTRING (26.1019276 44.4302748,",
                "26.1019382 44.4303191,26.1020002 44.4304202)"
            );
            assert_eq!(wkt, wkt_ok);
        });
    }

    #[test]
    fn test_feature_json() {
        with_feature("roads.geojson", 236194095, |feature| {
            let json = feature.geometry().unwrap().json();
            let json_ok = format!(
                "{}{}{}{}",
                "{ \"type\": \"LineString\", \"coordinates\": [ ",
                "[ 26.1019276, 44.4302748 ], ",
                "[ 26.1019382, 44.4303191 ], ",
                "[ 26.1020002, 44.4304202 ] ] }"
            );
            assert_eq!(json.unwrap(), json_ok);
        });
    }

    #[test]
    fn test_write_features() -> Result<()> {
        use std::fs;

        let name_idx = 0;
        let value_idx = 1;
        let int_value_idx = 2;

        {
            let driver = DriverManager::get_driver_by_name("GeoJSON").unwrap();
            let mut ds = driver
                .create_vector_only(fixture("output.geojson"))
                .unwrap();
            let layer = ds.create_layer(Default::default()).unwrap();
            layer.create_defn_fields(&[
                ("Name", OGRFieldType::OFTString),
                ("Value", OGRFieldType::OFTReal),
                ("Int_value", OGRFieldType::OFTInteger),
            ])?;

            let mut feature = Feature::new(layer.defn())?;
            let geometry = Geometry::from_wkt("POINT (1 2)")?;
            feature.set_geometry(geometry)?;
            feature.set_field_string(name_idx, "Feature 1")?;
            feature.set_field_double(value_idx, 45.78)?;
            feature.set_field_integer(int_value_idx, 1)?;
            feature.create(&layer)?;

            // dataset is closed here
        }

        {
            let ds = Dataset::open(fixture("output.geojson")).unwrap();
            let mut layer = ds.layer(0).unwrap();
            assert_eq!(
                layer.defn.geometry_type(),
                gdal_sys::OGRwkbGeometryType::wkbPoint
            );
            let ft = layer.features().next().unwrap();
            assert_eq!(ft.geometry().unwrap().wkt().unwrap(), "POINT (1 2)");
            assert_eq!(
                ft.field(name_idx).unwrap().unwrap().into_string(),
                Some("Feature 1".to_string())
            );
            assert_eq!(
                ft.field(value_idx).unwrap().unwrap().into_real(),
                Some(45.78)
            );
            assert_eq!(
                ft.field(int_value_idx).unwrap().unwrap().into_int(),
                Some(1)
            );
        }
        fs::remove_file(fixture("output.geojson")).unwrap();

        Ok(())
    }

    #[test]
    fn test_features_reset() {
        with_layer("roads.geojson", |mut layer| {
            assert_eq!(layer.features().count(), layer.features().count(),);
        });
    }

    #[test]
    fn test_set_attribute_filter() {
        with_layer("roads.geojson", |mut layer| {
            // check number without calling any function
            assert_eq!(layer.features().count(), 21);

            // check if clearing does not corrupt anything
            layer.clear_attribute_filter();
            assert_eq!(layer.features().count(), 21);

            // apply actual filter
            layer.set_attribute_filter("highway = 'primary'").unwrap();

            assert_eq!(layer.features().count(), 1);
            let highway_idx = layer.defn().field_index("highway").unwrap();
            assert_eq!(
                layer
                    .features()
                    .next()
                    .unwrap()
                    .field_as_string(highway_idx)
                    .unwrap()
                    .unwrap(),
                "primary"
            );

            // clearing and check again
            layer.clear_attribute_filter();

            assert_eq!(layer.features().count(), 21);

            {
                let _nolog = SuppressGDALErrorLog::new();
                // force error
                assert!(matches!(
                    layer.set_attribute_filter("foo = bar").unwrap_err(),
                    GdalError::OgrError {
                        err: gdal_sys::OGRErr::OGRERR_CORRUPT_DATA,
                        method_name: "OGR_L_SetAttributeFilter",
                    }
                ));
            }
        });
    }

    #[test]
    fn test_set_feature() {
        let ds_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_UPDATE,
            ..DatasetOptions::default()
        };
        let tmp_file = TempFixture::empty("test.s3db");
        std::fs::copy(fixture("three_layer_ds.s3db"), &tmp_file).unwrap();
        let ds = Dataset::open_ex(&tmp_file, ds_options).unwrap();
        let mut layer = ds.layer(0).unwrap();
        let fids: Vec<u64> = layer.features().map(|f| f.fid().unwrap()).collect();
        let mut feature = layer.feature(fids[0]).unwrap();
        let id_index = feature.field_index("id").unwrap();
        // to original value of the id field in fid 0 is null; we will set it to 1.
        feature.set_field_integer(id_index, 1).ok();
        layer.set_feature(feature).ok();

        // now we check that the field is 1.
        let ds = Dataset::open(&tmp_file).unwrap();
        let layer = ds.layer(0).unwrap();
        let feature = layer.feature(fids[0]).unwrap();
        let value = feature
            .field(id_index)
            .unwrap()
            .unwrap()
            .into_int()
            .unwrap();
        assert_eq!(value, 1);
    }
    #[test]
    fn test_schema() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        // The layer name is "roads" in GDAL 2.2
        assert!(layer.name() == "OGRGeoJSON" || layer.name() == "roads");
        let name_list = layer
            .defn()
            .fields()
            .map(|f| (f.name(), f.field_type()))
            .collect::<Vec<_>>();
        let ok_names_types = [
            ("kind", OGRFieldType::OFTString),
            ("sort_key", OGRFieldType::OFTReal),
            ("is_link", OGRFieldType::OFTString),
            ("is_tunnel", OGRFieldType::OFTString),
            ("is_bridge", OGRFieldType::OFTString),
            ("railway", OGRFieldType::OFTString),
            ("highway", OGRFieldType::OFTString),
        ]
        .into_iter()
        .map(|s| (s.0.to_string(), s.1))
        .collect::<Vec<_>>();
        assert_eq!(name_list, ok_names_types);

        let field = layer.defn().fields().next().unwrap();
        assert_eq!(field.alternative_name(), "");
        assert_eq!(field.width(), 0);
        assert_eq!(field.precision(), 0);
        assert!(field.is_nullable());
        assert!(!field.is_unique());
        assert_eq!(field.default_value(), None);
    }

    #[test]
    fn test_geom_fields() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        let name_list = layer
            .defn()
            .geom_fields()
            .map(|f| (f.name(), f.field_type()))
            .collect::<Vec<_>>();
        let ok_names_types = [("", OGRwkbGeometryType::wkbLineString)]
            .into_iter()
            .map(|s| (s.0.to_string(), s.1))
            .collect::<Vec<_>>();
        assert_eq!(name_list, ok_names_types);

        let geom_field = layer.defn().geom_fields().next().unwrap();
        let mut spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
        spatial_ref2.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);

        assert_eq!(geom_field.spatial_ref().unwrap(), spatial_ref2);
    }

    #[test]
    fn test_two_geom_fields() -> Result<()> {
        let ds = Dataset::open(fixture("two_geoms.csv"))?;
        let mut layer = ds.layer(0)?;

        let geom_field_2_idx = layer
            .defn()
            .geometry_field_index("geom__WKTanother_geometry")
            .unwrap();
        assert_eq!(geom_field_2_idx, 1);

        let feature = layer.features().next().unwrap();
        let geom_1 = feature.geometry_by_index(0)?;
        let geom_2 = feature.geometry_by_index(1)?;
        assert_eq!(geom_1.get_point(0), (1.0, 2.0, 0.0));
        assert_eq!(geom_2.get_point(0), (10.0, 20.0, 0.0));

        Ok(())
    }

    #[test]
    fn test_get_layer_by_name() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        // The layer name is "roads" in GDAL 2.2
        if let Ok(layer) = ds.layer_by_name("OGRGeoJSON") {
            assert_eq!(layer.name(), "OGRGeoJSON");
        }
        if let Ok(layer) = ds.layer_by_name("roads") {
            assert_eq!(layer.name(), "roads");
        }
    }

    #[test]
    fn test_spatial_filter() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let mut layer = ds.layer(0).unwrap();
        assert_eq!(layer.features().count(), 21);

        let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
        layer.set_spatial_filter(&bbox);
        assert_eq!(layer.features().count(), 7);

        layer.clear_spatial_filter();
        assert_eq!(layer.features().count(), 21);

        // test filter as rectangle
        layer.set_spatial_filter_rect(26.1017, 44.4297, 26.1025, 44.4303);
        assert_eq!(layer.features().count(), 7);
    }
}
