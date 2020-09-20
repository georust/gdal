use crate::{
    Dataset, DatasetCommon, OGRwkbGeometryType, SpatialRef, SpatialRefCommon, SpatialRef_3_0,
    VectorDatasetCommon,
};
use std::path::Path;

macro_rules! fixture {
    ($name:expr) => {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
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

#[test]
fn test_geom_fields() {
    let mut ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();
    let name_list = layer
        .defn()
        .geom_fields()
        .map(|f| (f.name(), f.field_type()))
        .collect::<Vec<_>>();
    let ok_names_types = vec![("", OGRwkbGeometryType::wkbLineString)]
        .iter()
        .map(|s| (s.0.to_string(), s.1))
        .collect::<Vec<_>>();
    assert_eq!(name_list, ok_names_types);

    let geom_field = layer.defn().geom_fields().next().unwrap();
    let spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();

    spatial_ref2.set_axis_mapping_strategy(0);

    assert!(geom_field.spatial_ref().unwrap() == spatial_ref2);
}
