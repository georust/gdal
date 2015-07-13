use std::ptr::null;
use vector::{ogr, Dataset, Feature, Geometry};
use vector::defn::Defn;

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
    defn: Defn,
}


impl<'a> Layer<'a> {
    pub unsafe fn _with_dataset(dataset: &'a Dataset, c_layer: *const ()) -> Layer<'a> {
        let c_defn = ogr::OGR_L_GetLayerDefn(c_layer);
        let defn = Defn::_with_c_defn(c_defn);
        return Layer{_dataset: dataset, c_layer: c_layer, defn: defn};
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

    pub fn defn(&'a self) -> &'a Defn {
        &self.defn
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
