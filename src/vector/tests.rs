use std::path::Path;
use super::{Dataset, Feature, FeatureIterator, Geometry, OwnedGeometry};


fn fixtures() -> Path {
    return Path::new(file!()).dir_path().dir_path().dir_path().join("fixtures");
}


fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}


#[test]
fn test_layer_count() {
    let ds = Dataset::open(&fixtures().join("roads.geojson")).unwrap();
    assert_eq!(ds.count(), 1);
}


fn with_features<F>(fixture: &str, f: F) where F: Fn(FeatureIterator) {
    let ds = Dataset::open(&fixtures().join(fixture)).unwrap();
    let layer = ds.layer(0).unwrap();
    f(layer.features());
}


fn with_first_feature<F>(fixture: &str, f: F) where F: Fn(Feature) {
    with_features(fixture, |mut features| f(features.next().unwrap()));
}


#[test]
fn test_iterate_features() {
    with_features("roads.geojson", |features| {
        let feature_vec: Vec<Feature> = features.collect();
        assert_eq!(feature_vec.len(), 21);
    });
}


#[test]
fn test_string_field() {
    with_features("roads.geojson", |mut features| {
        let feature = features.next().unwrap();
        assert_eq!(feature.field("highway")
                          .unwrap()
                          .as_string(),
                   "footway".to_string());
        assert_eq!(
            features.filter(|field| {
                let highway = field.field("highway")
                                   .unwrap()
                                   .as_string();
                highway == "residential".to_string() })
                .count(),
            2);
    });
}


#[test]
fn test_float_field() {
    with_first_feature("roads.geojson", |feature| {
        assert_almost_eq(
            feature.field("sort_key")
                   .unwrap()
                   .as_real(),
            -9.0
        );
    });
}


#[test]
fn test_missing_field() {
    with_first_feature("roads.geojson", |feature| {
        assert!(feature.field("no such field").is_none());
    });
}


#[test]
fn test_wkt() {
    with_first_feature("roads.geojson", |feature| {
        let wkt = feature.wkt();
        let wkt_ok = format!("{}{}",
            "LINESTRING (26.1019276 44.4302748,",
            "26.1019382 44.4303191,26.1020002 44.4304202)"
            ).to_string();
        assert_eq!(wkt, wkt_ok);
    });
}


#[test]
fn test_json() {
    with_first_feature("roads.geojson", |feature| {
        let json = feature.json();
        let json_ok = format!("{}{}{}{}",
            "{ \"type\": \"LineString\", \"coordinates\": [ ",
            "[ 26.1019276, 44.4302748 ], ",
            "[ 26.1019382, 44.4303191 ], ",
            "[ 26.1020002, 44.4304202 ] ] }"
            ).to_string();
        assert_eq!(json, json_ok);
    });
}


#[test]
fn test_schema() {
    let ds = Dataset::open(&fixtures().join("roads.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();
    let name_list: Vec<String> = layer
        .fields()
        .map(|f| f.name())
        .collect();
    let ok_names: Vec<String> = vec!(
        "kind", "sort_key", "is_link", "is_tunnel",
        "is_bridge", "railway", "highway")
        .iter().map(|s| s.to_string()).collect();
    assert_eq!(name_list, ok_names);
}

#[test]
fn test_create_bbox() {
    let bbox = OwnedGeometry::bbox(-27., 33., 52., 85.);
    assert_eq!(bbox.json(), "{ \"type\": \"Polygon\", \"coordinates\": [ [ [ -27.0, 85.0 ], [ 52.0, 85.0 ], [ 52.0, 33.0 ], [ -27.0, 33.0 ], [ -27.0, 85.0 ] ] ] }");
}

#[test]
fn test_spatial_filter() {
    let ds = Dataset::open(&fixtures().join("roads.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();

    let all_features: Vec<Feature> = layer.features().collect();
    assert_eq!(all_features.len(), 21);

    let bbox = OwnedGeometry::bbox(26.1017, 44.4297, 26.1025, 44.4303);
    layer.set_spatial_filter(&bbox);

    let some_features: Vec<Feature> = layer.features().collect();
    assert_eq!(some_features.len(), 7);

    layer.clear_spatial_filter();

    let again_all_features: Vec<Feature> = layer.features().collect();
    assert_eq!(again_all_features.len(), 21);
}
