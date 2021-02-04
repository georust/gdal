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

/// Layer in a vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::Dataset;
///
/// let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// let mut layer = dataset.layer(0).unwrap();
/// for feature in layer.features() {
///     // do something with each feature
/// }
/// ```
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

impl<'a> Layer<'a> {
    /// Creates a new Layer from a GDAL layer pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_layer(_: &'a Dataset, c_layer: OGRLayerH) -> Layer<'a> {
        let c_defn = gdal_sys::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::from_c_defn(c_defn);
        Layer {
            c_layer,
            defn,
            phantom: PhantomData,
        }
    }

    /// Returns the C wrapped pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_layer(&self) -> OGRLayerH {
        self.c_layer
    }

    /// Returns the feature with the given feature id `fid`, or `None` if not found.
    ///
    /// This function is unaffected by the spatial or attribute filters.
    ///
    /// Not all drivers support this efficiently; however, the call should always work if the
    /// feature exists, as a fallback implementation just scans all the features in the layer
    /// looking for the desired feature.
    ///
    /// **Important**: do not call `feature` while iterating over features using the
    /// [`features`](Layer::features) function. Even if you don't mutate anything, this can cause
    /// the the iterator to get interrupted. This is a limitation of the GDAL library.
    pub fn feature(&self, fid: u64) -> Option<Feature> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetFeature(self.c_layer, fid as i64) };
        if c_feature.is_null() {
            None
        } else {
            Some(unsafe { Feature::from_c_feature(self.defn(), c_feature) })
        }
    }

    /// Iterate over all features in this layer.
    pub fn features(&mut self) -> FeatureIterator {
        self.reset_feature_reading();
        FeatureIterator::_with_layer(self)
    }

    pub fn set_spatial_filter(&mut self, geometry: &Geometry) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer, geometry.c_geometry()) };
    }

    pub fn clear_spatial_filter(&mut self) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer, null_mut()) };
    }

    /// Get the name of this layer.
    pub fn name(&self) -> String {
        let rv = unsafe { gdal_sys::OGR_L_GetName(self.c_layer) };
        _string(rv)
    }

    pub fn defn(&self) -> &Defn {
        &self.defn
    }

    pub fn create_defn_fields(&self, fields_def: &[(&str, OGRFieldType::Type)]) -> Result<()> {
        for fd in fields_def {
            let fdefn = FieldDefn::new(fd.0, fd.1)?;
            fdefn.add_to_layer(self)?;
        }
        Ok(())
    }
    pub fn create_feature(&mut self, geometry: Geometry) -> Result<()> {
        let c_feature = unsafe { gdal_sys::OGR_F_Create(self.defn.c_defn()) };
        let c_geometry = unsafe { geometry.into_c_geometry() };
        let rv = unsafe { gdal_sys::OGR_F_SetGeometryDirectly(c_feature, c_geometry) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_F_SetGeometryDirectly",
            });
        }
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(self.c_layer, c_feature) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            });
        }
        Ok(())
    }

    pub fn create_feature_fields(
        &mut self,
        geometry: Geometry,
        field_names: &[&str],
        values: &[FieldValue],
    ) -> Result<()> {
        let mut ft = Feature::new(&self.defn)?;
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
    pub fn feature_count(&self) -> u64 {
        (unsafe { gdal_sys::OGR_L_GetFeatureCount(self.c_layer, 1) }) as u64
    }

    /// Returns the number of features in this layer, if it is possible to compute this
    /// efficiently.
    ///
    /// For some drivers, it would be expensive to establish the feature count, in which case
    /// [`None`] will be returned.
    ///
    /// The returned count takes the [spatial filter](`Layer::set_spatial_filter`) into account.
    /// For dynamic databases the count may not be exact.
    pub fn try_feature_count(&self) -> Option<u64> {
        let rv = unsafe { gdal_sys::OGR_L_GetFeatureCount(self.c_layer, 0) };
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
    pub fn get_extent(&self) -> Result<gdal_sys::OGREnvelope> {
        let mut envelope = OGREnvelope {
            MinX: 0.0,
            MaxX: 0.0,
            MinY: 0.0,
            MaxY: 0.0,
        };
        let force = 1;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer, &mut envelope, force) };
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
    pub fn try_get_extent(&self) -> Result<Option<gdal_sys::OGREnvelope>> {
        let mut envelope = OGREnvelope {
            MinX: 0.0,
            MaxX: 0.0,
            MinY: 0.0,
            MaxY: 0.0,
        };
        let force = 0;
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer, &mut envelope, force) };
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

    pub fn spatial_ref(&self) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OGR_L_GetSpatialRef(self.c_layer) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_L_GetSpatialRef"));
        }
        SpatialRef::from_c_obj(c_obj)
    }

    fn reset_feature_reading(&mut self) {
        unsafe {
            gdal_sys::OGR_L_ResetReading(self.c_layer);
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
    pub fn _with_layer(layer: &'a Layer) -> FeatureIterator<'a> {
        let defn = layer.defn();
        let size_hint = layer
            .try_feature_count()
            .map(|s| s.try_into().ok())
            .flatten();
        FeatureIterator { c_layer: layer.c_layer, size_hint, defn }
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
    pub fn add_to_layer(&self, layer: &Layer) -> Result<()> {
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
