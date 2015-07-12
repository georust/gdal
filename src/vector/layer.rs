use std::ptr::null;
use libc::c_int;
use utils::_string;
use vector::{ogr, Dataset, Feature, Geometry};

/// Layer in a vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::vector::Dataset;
///
/// let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// let layer = dataset.layer(0).unwrap();
/// for feature in layer.features() {
///     // do something with each feature
/// }
/// ```
pub struct Layer<'a> {
    _dataset: &'a Dataset,
    c_layer: *const (),
}


impl<'a> Layer<'a> {
    pub unsafe fn _with_dataset(dataset: &'a Dataset, c_layer: *const ()) -> Layer<'a> {
        return Layer{_dataset: dataset, c_layer: c_layer};
    }

    /// Iterate over the field schema of this layer.
    pub fn fields(&'a self) -> FieldIterator<'a> {
        let c_feature_defn = unsafe { ogr::OGR_L_GetLayerDefn(self.c_layer) };
        let total = unsafe { ogr::OGR_FD_GetFieldCount(c_feature_defn) } as isize;
        return FieldIterator{
            layer: self,
            c_feature_defn: c_feature_defn,
            next_id: 0,
            total: total
        };
    }

    /// Iterate over all features in this layer.
    pub fn features(&'a self) -> FeatureIterator<'a> {
        return FeatureIterator::_with_layer(self);
    }

    pub fn set_spatial_filter(&'a self, geometry: &Geometry) {
        unsafe { ogr::OGR_L_SetSpatialFilter(self.c_layer, geometry.c_geometry()) };
    }

    pub fn clear_spatial_filter(&'a self) {
        unsafe { ogr::OGR_L_SetSpatialFilter(self.c_layer, null()) };
    }
}


pub struct FieldIterator<'a> {
    layer: &'a Layer<'a>,
    c_feature_defn: *const (),
    next_id: isize,
    total: isize,
}


impl<'a> Iterator for FieldIterator<'a> {
    type Item = Field<'a>;

    #[inline]
    fn next(&mut self) -> Option<Field<'a>> {
        if self.next_id == self.total {
            return None;
        }
        let field = Field{
            _layer: self.layer,
            c_field_defn: unsafe { ogr::OGR_FD_GetFieldDefn(
                self.c_feature_defn,
                self.next_id as c_int
            ) }
        };
        self.next_id += 1;
        return Some(field);
    }
}


pub struct Field<'a> {
    _layer: &'a Layer<'a>,
    c_field_defn: *const (),
}


impl<'a> Field<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { ogr::OGR_Fld_GetNameRef(self.c_field_defn) };
        return _string(rv);
    }
}


pub struct FeatureIterator<'a> {
    layer: &'a Layer<'a>,
}


impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { ogr::OGR_L_GetNextFeature(self.layer.c_layer) };
        return match c_feature.is_null() {
            true  => None,
            false => Some(unsafe { Feature::_with_layer(self.layer, c_feature) }),
        };
    }
}


impl<'a> FeatureIterator<'a> {
    pub fn _with_layer(layer: &'a Layer<'a>) -> FeatureIterator<'a> {
        return FeatureIterator{layer: layer};
    }
}
