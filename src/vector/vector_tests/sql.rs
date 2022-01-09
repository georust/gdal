use std::collections::HashSet;

use crate::{
    fixture,
    vector::{sql, Geometry, LayerAccess},
    Dataset,
};

#[test]
fn test_sql() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'pedestrian'";
    let mut result_set = ds
        .execute_sql(query, None, sql::Dialect::DEFAULT)
        .unwrap()
        .unwrap();

    let field_names: HashSet<_> = result_set
        .defn()
        .fields()
        .map(|field| field.name())
        .collect();

    let mut correct_field_names = HashSet::new();
    correct_field_names.insert("kind".into());
    correct_field_names.insert("is_bridge".into());
    correct_field_names.insert("highway".into());

    assert_eq!(correct_field_names, field_names);
    assert_eq!(10, result_set.feature_count());

    for feature in result_set.features() {
        let highway = feature
            .field("highway")
            .unwrap()
            .unwrap()
            .into_string()
            .unwrap();

        assert_eq!("pedestrian", highway);
    }
}

#[test]
fn test_sql_with_spatial_filter() {
    let query = "SELECT * FROM roads WHERE highway = 'pedestrian'";
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
    let mut result_set = ds
        .execute_sql(query, Some(&bbox), sql::Dialect::DEFAULT)
        .unwrap()
        .unwrap();

    assert_eq!(2, result_set.feature_count());
    let mut correct_fids = HashSet::new();
    correct_fids.insert(252725993);
    correct_fids.insert(23489656);

    let mut fids = HashSet::new();
    for feature in result_set.features() {
        let highway = feature
            .field("highway")
            .unwrap()
            .unwrap()
            .into_string()
            .unwrap();

        assert_eq!("pedestrian", highway);
        fids.insert(feature.fid().unwrap());
    }

    assert_eq!(correct_fids, fids);
}

#[test]
fn test_sql_with_dialect() {
    let query = "SELECT * FROM roads WHERE highway = 'pedestrian' and NumPoints(GEOMETRY) = 3";
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
    let mut result_set = ds
        .execute_sql(query, Some(&bbox), sql::Dialect::SQLITE)
        .unwrap()
        .unwrap();

    assert_eq!(1, result_set.feature_count());
    let mut features: Vec<_> = result_set.features().collect();
    let feature = features.pop().unwrap();
    let highway = feature
        .field("highway")
        .unwrap()
        .unwrap()
        .into_string()
        .unwrap();

    assert_eq!("pedestrian", highway);
}

#[test]
fn test_sql_empty_result() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'jazz hands 👐'";
    let mut result_set = ds
        .execute_sql(query, None, sql::Dialect::DEFAULT)
        .unwrap()
        .unwrap();
    assert_eq!(0, result_set.feature_count());
    assert_eq!(0, result_set.features().count());
}

#[test]
fn test_sql_no_result() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let query = "ALTER TABLE roads ADD COLUMN fun integer";
    let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT).unwrap();
    assert!(result_set.is_none());
}

#[test]
fn test_sql_bad_query() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();

    let query = "SELECT nope FROM roads";
    let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
    assert!(result_set.is_err());

    let query = "SELECT nope FROM";
    let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
    assert!(result_set.is_err());

    let query = "SELECT ninetynineredballoons(highway) FROM roads";
    let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
    assert!(result_set.is_err());
}
