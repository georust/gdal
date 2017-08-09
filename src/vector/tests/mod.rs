use std::path::Path;
use super::{Driver, Dataset, Feature, FeatureIterator, FieldValue, Geometry, OGRFieldType, WkbType};

mod convert_geo;

macro_rules! fixture {
    ($name:expr) => (
        Path::new(file!())
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join("fixtures").as_path()
            .join($name).as_path()
    )
}


fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}


#[test]
fn test_layer_count() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    assert_eq!(ds.count(), 1);
}


fn with_features<F>(name: &str, f: F) where F: Fn(FeatureIterator) {
    let mut ds = Dataset::open(fixture!(name)).unwrap();
    let layer = ds.layer(0).unwrap();
    f(layer.features());
}


fn with_first_feature<F>(name: &str, f: F) where F: Fn(Feature) {
    with_features(name, |mut features| f(features.next().unwrap()));
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
                          .to_string(),
                   Some("footway".to_string()));
        assert_eq!(
            features.filter(|field| {
                let highway = field.field("highway")
                                   .unwrap()
                                   .to_string();
                highway == Some("residential".to_string()) })
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
                   .to_real()
                   .unwrap(),
            -9.0
        );
    });
}


#[test]
fn test_missing_field() {
    with_first_feature("roads.geojson", |feature| {
        assert!(feature.field("no such field").is_err());
    });
}


#[test]
fn test_geom_accessors() {
    with_first_feature("roads.geojson", |feature| {
        let geom = feature.geometry();
        assert_eq!(geom.geometry_type(), WkbType::WkbLinestring);
        let coords = geom.get_point_vec();
        assert_eq!(coords, [(26.1019276, 44.4302748, 0.0), (26.1019382, 44.4303191, 0.0), (26.1020002, 44.4304202, 0.0)]);
        assert_eq!(geom.geometry_count(), 0);
    });
}


#[test]
fn test_wkt() {
    with_first_feature("roads.geojson", |feature| {
        let wkt = feature.geometry().wkt().unwrap();
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
        let json = feature.geometry().json();
        let json_ok = format!("{}{}{}{}",
            "{ \"type\": \"LineString\", \"coordinates\": [ ",
            "[ 26.1019276, 44.4302748 ], ",
            "[ 26.1019382, 44.4303191 ], ",
            "[ 26.1020002, 44.4304202 ] ] }"
            ).to_string();
        assert_eq!(json.unwrap(), json_ok);
    });
}


#[test]
fn test_schema() {
    let mut ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();
    assert_eq!(layer.name(), "OGRGeoJSON".to_string());
    let name_list: Vec<(String, OGRFieldType)> = layer
        .defn().fields()
        .map(|f| (f.name(), f.field_type()))
        .collect();
    let ok_names_types: Vec<(String, OGRFieldType)> = vec!(
        ("id", OGRFieldType::OFTString),
        ("kind", OGRFieldType::OFTString),
        ("sort_key",  OGRFieldType::OFTReal),
        ("is_link", OGRFieldType::OFTString),
        ("is_tunnel", OGRFieldType::OFTString),
        ("is_bridge", OGRFieldType::OFTString),
        ("railway", OGRFieldType::OFTString),
        ("highway", OGRFieldType::OFTString))
        .iter().map(|s| (s.0.to_string(), s.1)).collect();
    assert_eq!(name_list, ok_names_types);
}

#[test]
fn test_get_layer_by_name() {
    let mut ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let layer = ds.layer_by_name("OGRGeoJSON").unwrap();
    assert_eq!(layer.name(), "OGRGeoJSON");
}

#[test]
fn test_create_bbox() {
    let bbox = Geometry::bbox(-27., 33., 52., 85.).unwrap();
    assert_eq!(bbox.json().unwrap(), "{ \"type\": \"Polygon\", \"coordinates\": [ [ [ -27.0, 85.0 ], [ 52.0, 85.0 ], [ 52.0, 33.0 ], [ -27.0, 33.0 ], [ -27.0, 85.0 ] ] ] }");
}

#[test]
fn test_spatial_filter() {
    let mut ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();

    let all_features: Vec<Feature> = layer.features().collect();
    assert_eq!(all_features.len(), 21);

    let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
    layer.set_spatial_filter(&bbox);

    let some_features: Vec<Feature> = layer.features().collect();
    assert_eq!(some_features.len(), 7);

    layer.clear_spatial_filter();

    let again_all_features: Vec<Feature> = layer.features().collect();
    assert_eq!(again_all_features.len(), 21);
}

#[test]
fn test_convex_hull() {
    let star = "POLYGON ((0 1,3 1,1 3,1.5 0.0,2 3,0 1))";
    let hull = "POLYGON ((1.5 0.0,0 1,1 3,2 3,3 1,1.5 0.0))";
    assert_eq!(Geometry::from_wkt(star).unwrap().convex_hull().unwrap().wkt().unwrap(), hull);
}

#[test]
fn test_write_features() {
    use std::fs;

    {
        let driver = Driver::get("GeoJSON").unwrap();
        let mut ds = driver.create(fixture!("output.geojson")).unwrap();
        let mut layer = ds.create_layer().unwrap();
        layer.create_defn_fields(&[("Name",  OGRFieldType::OFTString), ("Value",  OGRFieldType::OFTReal), ("Int_value", OGRFieldType::OFTInteger)]);
        layer.create_feature_fields(
            Geometry::from_wkt("POINT (1 2)").unwrap(), &["Name", "Value", "Int_value"],
            &[FieldValue::StringValue("Feature 1".to_string()), FieldValue::RealValue(45.78), FieldValue::IntegerValue(1)]
            ).unwrap();
        // dataset is closed here
    }

    let mut ds = Dataset::open(fixture!("output.geojson")).unwrap();
    fs::remove_file(fixture!("output.geojson")).unwrap();
    let layer = ds.layer(0).unwrap();
    let ft = layer.features().next().unwrap();
    assert_eq!(ft.geometry().wkt().unwrap(), "POINT (1 2)");
    assert_eq!(ft.field("Name").unwrap().to_string(), Some("Feature 1".to_string()));
    assert_eq!(ft.field("Value").unwrap().to_real(), Some(45.78));
    assert_eq!(ft.field("Int_value").unwrap().to_int(), Some(1));
}
