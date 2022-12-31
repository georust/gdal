use crate::spatial_ref::SpatialRef;
use crate::test_utils::{fixture, TempFixture};
use crate::{assert_almost_eq, Dataset, DatasetOptions, GdalOpenFlags};

use super::{
    Feature, FeatureIterator, FieldValue, Geometry, Layer, LayerAccess, LayerCaps::*, OGRFieldType,
    OGRwkbGeometryType, OwnedLayer,
};

mod convert_geo;
mod sql;

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
    assert!(layer.try_get_extent().unwrap().is_none());
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
    assert!(!layer.has_capability(OLCFastGetExtent));
    assert!(layer.has_capability(OLCRandomRead));
    assert!(layer.has_capability(OLCStringsAsUTF8));
}

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

#[cfg(test)]
mod tests {
    use gdal_sys::OGRwkbGeometryType::{wkbLineString, wkbLinearRing, wkbPolygon};

    use crate::test_utils::SuppressGDALErrorLog;
    use crate::{
        errors::{GdalError, Result},
        DriverManager,
    };

    use super::*;

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
            assert_eq!(feature.field("id").unwrap(), None);
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
            assert_eq!(
                feature.field("highway").unwrap().unwrap().into_string(),
                Some("footway".to_string())
            );
        });
        with_features("roads.geojson", |features| {
            assert_eq!(
                features
                    .filter(|field| {
                        let highway = field.field("highway").unwrap().unwrap().into_string();
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
            assert_eq!(
                feature.field("some_int").unwrap(),
                Some(FieldValue::IntegerValue(0))
            );
            assert_eq!(feature.field("some_string").unwrap(), None);
        });
    }

    #[test]
    fn test_string_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(
                feature.field("a_string_list").unwrap().unwrap(),
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
            let feature = features.next().unwrap();
            let value = FieldValue::StringListValue(vec![
                String::from("the"),
                String::from("new"),
                String::from("strings"),
            ]);
            feature.set_field("a_string_list", &value).unwrap();
            assert_eq!(feature.field("a_string_list").unwrap().unwrap(), value);
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_get_field_as_x_by_name() {
        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();

            assert_eq!(
                feature.field_as_string_by_name("highway").unwrap(),
                Some("footway".to_owned())
            );

            assert_eq!(
                feature.field_as_string_by_name("sort_key").unwrap(),
                Some("-9".to_owned())
            );
            assert_eq!(
                feature.field_as_integer_by_name("sort_key").unwrap(),
                Some(-9)
            );
            assert_eq!(
                feature.field_as_integer64_by_name("sort_key").unwrap(),
                Some(-9)
            );
            assert_eq!(
                feature.field_as_double_by_name("sort_key").unwrap(),
                Some(-9.)
            );

            // test failed conversions
            assert_eq!(
                feature.field_as_integer_by_name("highway").unwrap(),
                Some(0)
            );
            assert_eq!(
                feature.field_as_integer64_by_name("highway").unwrap(),
                Some(0)
            );
            assert_eq!(
                feature.field_as_double_by_name("highway").unwrap(),
                Some(0.)
            );

            // test nulls
            assert_eq!(feature.field_as_string_by_name("railway").unwrap(), None);
            assert_eq!(feature.field_as_integer_by_name("railway").unwrap(), None);
            assert_eq!(feature.field_as_integer64_by_name("railway").unwrap(), None);
            assert_eq!(feature.field_as_double_by_name("railway").unwrap(), None);

            assert!(matches!(
                feature.field_as_string_by_name("not_a_field").unwrap_err(),
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

            assert_eq!(feature.field_as_datetime_by_name("dt").unwrap(), Some(dt));

            assert_eq!(feature.field_as_datetime(0).unwrap(), Some(dt));

            assert_eq!(feature.field_as_datetime_by_name("d").unwrap(), Some(d));

            assert_eq!(feature.field_as_datetime(1).unwrap(), Some(d));
        });

        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();

            let railway_field = 5;

            // test null
            assert_eq!(feature.field_as_datetime_by_name("railway").unwrap(), None);
            assert_eq!(feature.field_as_datetime(railway_field).unwrap(), None);

            // test error
            assert!(matches!(
                feature
                    .field_as_datetime_by_name("not_a_field")
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
            assert_eq!(feature.field("id").unwrap(), None);
        });
    }

    #[test]
    fn test_int_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(
                feature.field("an_int_list").unwrap().unwrap(),
                FieldValue::IntegerListValue(vec![1, 2])
            );
        });
    }

    #[test]
    fn test_set_int_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let value = FieldValue::IntegerListValue(vec![3, 4, 5]);
            feature.set_field("an_int_list", &value).unwrap();
            assert_eq!(feature.field("an_int_list").unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_real_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(
                feature.field("a_real_list").unwrap().unwrap(),
                FieldValue::RealListValue(vec![0.1, 0.2])
            );
        });
    }

    #[test]
    fn test_set_real_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let value = FieldValue::RealListValue(vec![2.5, 3.0, 4.75]);
            feature.set_field("a_real_list", &value).unwrap();
            assert_eq!(feature.field("a_real_list").unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_long_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(
                feature.field("a_long_list").unwrap().unwrap(),
                FieldValue::Integer64ListValue(vec![5000000000, 6000000000])
            );
        });
    }

    #[test]
    fn test_set_long_list_field() {
        with_features("soundg.json", |mut features| {
            let feature = features.next().unwrap();
            let value = FieldValue::Integer64ListValue(vec![7000000000, 8000000000]);
            feature.set_field("a_long_list", &value).unwrap();
            assert_eq!(feature.field("a_long_list").unwrap().unwrap(), value);
        });
    }

    #[test]
    fn test_float_field() {
        with_feature("roads.geojson", 236194095, |feature| {
            assert_almost_eq(
                feature
                    .field("sort_key")
                    .unwrap()
                    .unwrap()
                    .into_real()
                    .unwrap(),
                -9.0,
            );
        });
    }

    #[test]
    fn test_missing_field() {
        with_feature("roads.geojson", 236194095, |feature| {
            assert!(feature.field("no such field").is_err());
        });
    }

    #[test]
    fn test_geom_accessors() {
        with_feature("roads.geojson", 236194095, |feature| {
            let geom = feature.geometry().unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            let coords = geom.get_point_vec();
            assert_eq!(
                coords,
                [
                    (26.1019276, 44.4302748, 0.0),
                    (26.1019382, 44.4303191, 0.0),
                    (26.1020002, 44.4304202, 0.0)
                ]
            );
            assert_eq!(geom.geometry_count(), 0);

            let geom = feature.geometry_by_index(0).unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            assert!(feature.geometry_by_index(1).is_err());
            let geom = feature.geometry_by_name("");
            assert!(geom.is_ok());
            let geom = feature.geometry_by_name("").unwrap();
            assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbLineString);
            assert!(feature.geometry_by_name("FOO").is_err());
        });
    }

    #[test]
    fn test_ring_points() {
        let mut ring = Geometry::empty(wkbLinearRing).unwrap();
        ring.add_point_2d((1179091.1646903288, 712782.8838459781));
        ring.add_point_2d((1161053.0218226474, 667456.2684348812));
        ring.add_point_2d((1214704.933941905, 641092.8288590391));
        ring.add_point_2d((1228580.428455506, 682719.3123998424));
        ring.add_point_2d((1218405.0658121984, 721108.1805541387));
        ring.add_point_2d((1179091.1646903288, 712782.8838459781));
        assert!(!ring.is_empty());
        assert_eq!(ring.get_point_vec().len(), 6);
        let mut poly = Geometry::empty(wkbPolygon).unwrap();
        poly.add_geometry(ring.to_owned()).unwrap();
        // Points are in ring, not containing geometry.
        // NB: In Python SWIG bindings, `GetPoints` is fallible.
        assert!(poly.get_point_vec().is_empty());
        assert_eq!(poly.geometry_count(), 1);
        let ring_out = poly.get_geometry(0);
        // NB: `wkb()` shows it to be a `LINEARRING`, but returned type is LineString
        assert_eq!(ring_out.geometry_type(), wkbLineString);
        assert!(!&ring_out.is_empty());
        assert_eq!(ring.get_point_vec(), ring_out.get_point_vec());
    }

    #[test]
    fn test_get_inner_points() {
        let geom = Geometry::bbox(0., 0., 1., 1.).unwrap();
        assert!(!geom.is_empty());
        assert_eq!(geom.geometry_count(), 1);
        assert!(geom.area() > 0.);
        assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbPolygon);
        assert!(geom.json().unwrap().contains("Polygon"));
        let inner = geom.get_geometry(0);
        let points = inner.get_point_vec();
        assert!(!points.is_empty());
    }

    #[test]
    fn test_wkt() {
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
    fn test_json() {
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
        let ok_names_types = vec![
            ("kind", OGRFieldType::OFTString),
            ("sort_key", OGRFieldType::OFTReal),
            ("is_link", OGRFieldType::OFTString),
            ("is_tunnel", OGRFieldType::OFTString),
            ("is_bridge", OGRFieldType::OFTString),
            ("railway", OGRFieldType::OFTString),
            ("highway", OGRFieldType::OFTString),
        ]
        .iter()
        .map(|s| (s.0.to_string(), s.1))
        .collect::<Vec<_>>();
        assert_eq!(name_list, ok_names_types);
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
        let ok_names_types = vec![("", OGRwkbGeometryType::wkbLineString)]
            .iter()
            .map(|s| (s.0.to_string(), s.1))
            .collect::<Vec<_>>();
        assert_eq!(name_list, ok_names_types);

        let geom_field = layer.defn().geom_fields().next().unwrap();
        let spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
        #[cfg(major_ge_3)]
        spatial_ref2.set_axis_mapping_strategy(0);

        assert!(geom_field.spatial_ref().unwrap() == spatial_ref2);
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
    fn test_create_bbox() {
        let bbox = Geometry::bbox(-27., 33., 52., 85.).unwrap();
        assert_eq!(bbox.json().unwrap(), "{ \"type\": \"Polygon\", \"coordinates\": [ [ [ -27.0, 85.0 ], [ 52.0, 85.0 ], [ 52.0, 33.0 ], [ -27.0, 33.0 ], [ -27.0, 85.0 ] ] ] }");
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

    #[test]
    fn test_convex_hull() {
        let star = "POLYGON ((0 1,3 1,1 3,1.5 0.0,2 3,0 1))";
        let hull = "POLYGON ((1.5 0.0,0 1,1 3,2 3,3 1,1.5 0.0))";
        assert_eq!(
            Geometry::from_wkt(star)
                .unwrap()
                .convex_hull()
                .unwrap()
                .wkt()
                .unwrap(),
            hull
        );
    }

    #[test]
    #[cfg(any(all(major_is_2, minor_ge_1), major_ge_3))]
    fn test_delaunay_triangulation() -> Result<()> {
        let square = Geometry::from_wkt("POLYGON ((0 1,1 1,1 0,0 0,0 1))")?;
        let triangles = Geometry::from_wkt(
            "GEOMETRYCOLLECTION (POLYGON ((0 1,0 0,1 0,0 1)),POLYGON ((0 1,1 0,1 1,0 1)))",
        )?;
        assert_eq!(square.delaunay_triangulation(None)?, triangles);
        Ok(())
    }

    #[test]
    fn test_simplify() -> Result<()> {
        let line = Geometry::from_wkt("LINESTRING(1.2 0.19,1.63 0.58,1.98 0.65,2.17 0.89)")?;
        let triangles = Geometry::from_wkt("LINESTRING (1.2 0.19,2.17 0.89)")?;
        assert_eq!(line.simplify(0.5)?, triangles);
        Ok(())
    }

    #[test]
    fn test_simplify_preserve_topology() -> Result<()> {
        let donut = Geometry::from_wkt(
            "POLYGON ((20 35,10 30,10 10,30 5,45 20,20 35),(30 20,20 15,20 25,30 20))",
        )?;
        let triangles = Geometry::from_wkt(
            "POLYGON ((20 35,10 10,30 5,45 20,20 35),(30 20,20 15,20 25,30 20))",
        )?;
        assert_eq!(donut.simplify_preserve_topology(100.0)?, triangles);
        Ok(())
    }

    #[test]
    fn test_write_features() {
        use std::fs;

        {
            let driver = DriverManager::get_driver_by_name("GeoJSON").unwrap();
            let mut ds = driver
                .create_vector_only(fixture("output.geojson"))
                .unwrap();
            let mut layer = ds.create_layer(Default::default()).unwrap();
            layer
                .create_defn_fields(&[
                    ("Name", OGRFieldType::OFTString),
                    ("Value", OGRFieldType::OFTReal),
                    ("Int_value", OGRFieldType::OFTInteger),
                ])
                .unwrap();
            layer
                .create_feature_fields(
                    Geometry::from_wkt("POINT (1 2)").unwrap(),
                    &["Name", "Value", "Int_value"],
                    &[
                        FieldValue::StringValue("Feature 1".to_string()),
                        FieldValue::RealValue(45.78),
                        FieldValue::IntegerValue(1),
                    ],
                )
                .unwrap();
            // dataset is closed here
        }

        {
            let ds = Dataset::open(fixture("output.geojson")).unwrap();
            let mut layer = ds.layer(0).unwrap();
            let ft = layer.features().next().unwrap();
            assert_eq!(ft.geometry().unwrap().wkt().unwrap(), "POINT (1 2)");
            assert_eq!(
                ft.field("Name").unwrap().unwrap().into_string(),
                Some("Feature 1".to_string())
            );
            assert_eq!(ft.field("Value").unwrap().unwrap().into_real(), Some(45.78));
            assert_eq!(ft.field("Int_value").unwrap().unwrap().into_int(), Some(1));
        }
        fs::remove_file(fixture("output.geojson")).unwrap();
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
            assert_eq!(
                layer
                    .features()
                    .next()
                    .unwrap()
                    .field_as_string_by_name("highway")
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
        let feature = layer.feature(fids[0]).unwrap();
        // to original value of the id field in fid 0 is null; we will set it to 1.
        feature.set_field_integer("id", 1).ok();
        layer.set_feature(feature).ok();

        // now we check that the field is 1.
        let ds = Dataset::open(&tmp_file).unwrap();
        let layer = ds.layer(0).unwrap();
        let feature = layer.feature(fids[0]).unwrap();
        let value = feature.field("id").unwrap().unwrap().into_int().unwrap();
        assert_eq!(value, 1);
    }
}
