use crate::metadata::Metadata;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::defn::Defn;
use crate::vector::{Feature, FieldValue, Geometry};
use crate::{dataset::Dataset, gdal_major_object::MajorObject};
use gdal_sys::{
    self, GDALMajorObjectH, OGREnvelope, OGRErr, OGRFieldDefnH, OGRFieldType, OGRLayerH,
};
use libc::c_int;
use std::ptr::null_mut;
use std::{convert::TryInto, ffi::CString, marker::PhantomData};

use crate::errors::*;

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
    /// Layer capability for feature deletiond
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

impl<'a> MajorObject for Layer<'a> {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_layer
    }
}

impl<'a> Metadata for Layer<'a> {}

impl<'a> LayerAccess for Layer<'a> {
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
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
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

    /// Set a spatial filter on this layer.
    ///
    /// Refer [OGR_L_SetSpatialFilter](https://gdal.org/doxygen/classOGRLayer.html#a75c06b4993f8eb76b569f37365cd19ab)
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
        _string(rv)
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

    fn create_feature_fields(
        &mut self,
        geometry: Geometry,
        field_names: &[&str],
        values: &[FieldValue],
    ) -> Result<()> {
        let mut ft = Feature::new(self.defn())?;
        ft.set_geometry(geometry)?;
        for (fd, val) in field_names.iter().zip(values.iter()) {
            ft.set_field(fd, val)?;
        }
        ft.create(self)?;
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
    fn get_extent(&self) -> Result<gdal_sys::OGREnvelope> {
        let mut envelope = OGREnvelope {
            MinX: 0.0,
            MaxX: 0.0,
            MinY: 0.0,
            MaxY: 0.0,
        };
        let force = 1;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer(), &mut envelope, force) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_GetExtent",
            });
        }
        Ok(envelope)
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
    fn try_get_extent(&self) -> Result<Option<gdal_sys::OGREnvelope>> {
        let mut envelope = OGREnvelope {
            MinX: 0.0,
            MaxX: 0.0,
            MinY: 0.0,
            MaxY: 0.0,
        };
        let force = 0;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer(), &mut envelope, force) };
        if rv == OGRErr::OGRERR_FAILURE {
            Ok(None)
        } else {
            if rv != OGRErr::OGRERR_NONE {
                return Err(GdalError::OgrError {
                    err: rv,
                    method_name: "OGR_L_GetExtent",
                });
            }
            Ok(Some(envelope))
        }
    }

    /// Fetch the spatial reference system for this layer.
    ///
    /// Refer [OGR_L_GetSpatialRef](https://gdal.org/doxygen/classOGRLayer.html#a75c06b4993f8eb76b569f37365cd19ab)
    fn spatial_ref(&self) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OGR_L_GetSpatialRef(self.c_layer()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_L_GetSpatialRef"));
        }
        SpatialRef::from_c_obj(c_obj)
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
}

pub struct FeatureIterator<'a> {
    defn: &'a Defn,
    c_layer: OGRLayerH,
    size_hint: Option<usize>,
}

impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.c_layer) };
        if c_feature.is_null() {
            None
        } else {
            Some(unsafe { Feature::from_c_feature(self.defn, c_feature) })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_hint {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl<'a> FeatureIterator<'a> {
    pub(crate) fn _with_layer<L: LayerAccess>(layer: &'a L) -> Self {
        let defn = layer.defn();
        let size_hint = layer
            .try_feature_count()
            .map(|s| s.try_into().ok())
            .flatten();
        Self {
            c_layer: unsafe { layer.c_layer() },
            size_hint,
            defn,
        }
    }
}

pub struct OwnedFeatureIterator {
    pub(crate) layer: OwnedLayer,
    size_hint: Option<usize>,
}

impl<'a> Iterator for &'a mut OwnedFeatureIterator
where
    Self: 'a,
{
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.layer.c_layer()) };

        if c_feature.is_null() {
            return None;
        }

        Some(unsafe {
            // We have to convince the compiler that our `Defn` adheres to our iterator lifetime `<'a>`
            let defn: &'a Defn = std::mem::transmute::<&'_ _, &'a _>(self.layer.defn());

            Feature::from_c_feature(defn, c_feature)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_hint {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl OwnedFeatureIterator {
    pub(crate) fn _with_layer(layer: OwnedLayer) -> Self {
        let size_hint = layer
            .try_feature_count()
            .map(|s| s.try_into().ok())
            .flatten();
        Self { layer, size_hint }
    }

    pub fn into_layer(self) -> OwnedLayer {
        self.layer
    }
}

impl AsMut<OwnedFeatureIterator> for OwnedFeatureIterator {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl From<OwnedFeatureIterator> for OwnedLayer {
    fn from(feature_iterator: OwnedFeatureIterator) -> Self {
        feature_iterator.into_layer()
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
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
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
