use std::ptr::null;
use libc::{c_void};
use vector::{Feature, Geometry};
use vector::defn::Defn;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{ogr, ogr_enums};

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
    c_layer: *const c_void,
    defn: Defn,
}

impl MajorObject for Layer {
    unsafe fn gdal_object_ptr(&self) -> *const c_void {
        self.c_layer
    }
}

impl Metadata for Layer {}


impl Layer {
    pub unsafe fn _with_c_layer(c_layer: *const c_void) -> Layer {
        let c_defn = ogr::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::_with_c_defn(c_defn);
        return Layer{c_layer: c_layer, defn: defn};
    }

    /// Iterate over all features in this layer.
    pub fn features<'a>(&'a self) -> FeatureIterator<'a> {
        return FeatureIterator::_with_layer(&self);
    }

    pub fn set_spatial_filter(&self, geometry: &Geometry) {
        unsafe { ogr::OGR_L_SetSpatialFilter(self.c_layer, geometry.c_geometry()) };
    }

    pub fn clear_spatial_filter(&self) {
        unsafe { ogr::OGR_L_SetSpatialFilter(self.c_layer, null()) };
    }

    pub fn defn(&self) -> &Defn {
        &self.defn
    }

    pub fn create_feature(&mut self, geometry: Geometry) -> Result<()> {
        let c_feature = unsafe { ogr::OGR_F_Create(self.defn.c_defn()) };
        let c_geometry = unsafe { geometry.into_c_geometry() };
        let rv = unsafe { ogr::OGR_F_SetGeometryDirectly(c_feature, c_geometry) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_F_SetGeometryDirectly").into());
        }
        let rv = unsafe { ogr::OGR_L_CreateFeature(self.c_layer, c_feature) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_CreateFeature").into());
        }
        Ok(())
    }
}

pub struct FeatureIterator<'a> {
    layer: &'a Layer,
}

impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { ogr::OGR_L_GetNextFeature(self.layer.c_layer) };
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
