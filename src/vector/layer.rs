use std::ptr::null_mut;
use std::ffi::CString;
use libc::c_int;
use vector::{Feature, Geometry, FieldValue};
use vector::defn::Defn;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{self, OGRErr, OGREnvelope, OGRFieldDefnH, OGRFieldType, OGRLayerH, GDALMajorObjectH};
use utils::{_last_null_pointer_err, _string};
use spatial_ref::SpatialRef;

use errors::*;

/// Layer in a vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::vector::Dataset;
///
/// let mut dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// let layer = dataset.layer(0).unwrap();
/// for feature in layer.features() {
///     // do something with each feature
/// }
/// ```
pub struct Layer {
    c_layer: OGRLayerH,
    defn: Defn,
}

impl MajorObject for Layer {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_layer
    }
}

impl Metadata for Layer {}


impl Layer {
    pub unsafe fn _with_c_layer(c_layer: OGRLayerH) -> Layer {
        let c_defn = gdal_sys::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::_with_c_defn(c_defn);
        return Layer{c_layer: c_layer, defn: defn};
    }

    pub unsafe fn c_layer(&self) -> OGRLayerH {
        self.c_layer
    }

    /// Iterate over all features in this layer.
    pub fn features<'a>(&'a self) -> FeatureIterator<'a> {
        return FeatureIterator::_with_layer(&self);
    }

    pub fn set_spatial_filter(&self, geometry: &Geometry) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer, geometry.c_geometry()) };
    }

    pub fn clear_spatial_filter(&self) {
        unsafe { gdal_sys::OGR_L_SetSpatialFilter(self.c_layer, null_mut()) };
    }

    /// Get the name of this layer.
    pub fn name(&self) -> String {
        let rv = unsafe { gdal_sys::OGR_L_GetName(self.c_layer) };
        return _string(rv);
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
            return Err(ErrorKind::OgrError(rv, "OGR_F_SetGeometryDirectly").into());
        }
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(self.c_layer, c_feature) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_CreateFeature").into());
        }
        Ok(())
    }

    pub fn create_feature_fields(&mut self, geometry: Geometry,
                                 field_names: &[&str], values: &[FieldValue]) -> Result<()> {
        let mut ft = Feature::new(&self.defn)?;
        ft.set_geometry(geometry)?;
        for (fd, ref val) in field_names.iter().zip(values.iter()) {
            ft.set_field(fd, val)?;
        }
        ft.create(self)?;
        Ok(())
    }

    pub fn get_extent(&self, force: bool) -> Result<gdal_sys::OGREnvelope> {
        let mut envelope = OGREnvelope { MinX: 0.0, MaxX: 0.0, MinY: 0.0, MaxY: 0.0 };
        let force = if force { 1 } else { 0 };
        let rv = unsafe { gdal_sys::OGR_L_GetExtent(self.c_layer, &mut envelope, force) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_GetExtent").into());
        }
        Ok(envelope)
    }

    pub fn spatial_reference(&self) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OGR_L_GetSpatialRef(self.c_layer) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_L_GetSpatialRef").into());
        }
        SpatialRef::from_c_obj(c_obj)
    }
}

pub struct FeatureIterator<'a> {
    layer: &'a Layer,
}

impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.layer.c_layer) };
        return match c_feature.is_null() {
            true  => None,
            false => Some(unsafe { Feature::_with_c_feature(self.layer.defn(), c_feature) }),
        };
    }
}

impl<'a> FeatureIterator<'a> {
    pub fn _with_layer(layer: &'a Layer) -> FeatureIterator<'a> {
        return FeatureIterator{layer: layer};
    }
}

pub struct FieldDefn {
    c_obj: OGRFieldDefnH,
}

impl Drop for FieldDefn {
    fn drop(&mut self){
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
            return Err(_last_null_pointer_err("OGR_Fld_Create").into());
        };
        Ok(FieldDefn { c_obj: c_obj})
    }
    pub fn set_width(&self, width: i32) {
        unsafe {gdal_sys:: OGR_Fld_SetWidth(self.c_obj, width as c_int) };
    }
    pub fn set_precision(&self, precision: i32) {
        unsafe {gdal_sys:: OGR_Fld_SetPrecision(self.c_obj, precision as c_int) };
    }
    pub fn add_to_layer(&self, layer: &Layer) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_L_CreateField(layer.c_layer(), self.c_obj, 1) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_CreateFeature").into());
        }
        Ok(())
    }
}
