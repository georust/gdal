use gdal::errors::Result;
use gdal::vector::{Defn, Feature, FieldDefn, FieldValue, Geometry, LayerAccess, OGRFieldType};
use gdal::DriverManager;
use std::fs;

/// Example 1, the detailed way:
fn example_1() -> Result<()> {
    let path = std::env::temp_dir().join("output1.geojson");
    let _ = fs::remove_file(&path);
    let drv = DriverManager::get_driver_by_name("GeoJSON")?;
    let mut ds = drv.create_vector_only(path.to_str().unwrap())?;

    let lyr = ds.create_layer(Default::default())?;

    let field_defn = FieldDefn::new("Name", OGRFieldType::OFTString)?;
    field_defn.set_width(80);
    field_defn.add_to_layer(&lyr)?;

    let field_defn = FieldDefn::new("Value", OGRFieldType::OFTReal)?;
    field_defn.add_to_layer(&lyr)?;

    let defn = Defn::from_layer(&lyr);

    // 1st feature:
    let mut ft = Feature::new(&defn)?;
    ft.set_geometry(Geometry::from_wkt("POINT (45.21 21.76)")?)?;
    ft.set_field_string("Name", "Feature 1")?;
    ft.set_field_double("Value", 45.78)?;
    ft.create(&lyr)?;

    // 2nd feature:
    let mut ft = Feature::new(&defn)?;
    ft.set_field_double("Value", 0.789)?;
    ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)")?)?;
    ft.set_field_string("Name", "Feature 2")?;
    ft.create(&lyr)?;

    // Feature triggering an error due to a wrong field name:
    let mut ft = Feature::new(&defn)?;
    ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)")?)?;
    ft.set_field_string("Name", "Feature 2")?;
    match ft.set_field_double("Values", 0.789) {
        Ok(v) => v,
        Err(err) => println!("{err}"),
    };
    ft.create(&lyr)?;

    Ok(())
}

/// Example 2, same output, shortened way:
fn example_2() -> Result<()> {
    let path = std::env::temp_dir().join("output2.geojson");
    let _ = fs::remove_file(&path);
    let driver = DriverManager::get_driver_by_name("GeoJSON")?;
    let mut ds = driver.create_vector_only(path.to_str().unwrap())?;
    let mut layer = ds.create_layer(Default::default())?;

    layer.create_defn_fields(&[
        ("Name", OGRFieldType::OFTString),
        ("Value", OGRFieldType::OFTReal),
    ])?;

    layer.create_feature_fields(
        Geometry::from_wkt("POINT (45.21 21.76)")?,
        &["Name", "Value"],
        &[
            FieldValue::StringValue("Feature 1".to_string()),
            FieldValue::RealValue(45.78),
        ],
    )?;

    layer.create_feature_fields(
        Geometry::from_wkt("POINT (46.50 22.50)")?,
        &["Name", "Value"],
        &[
            FieldValue::StringValue("Feature 2".to_string()),
            FieldValue::RealValue(0.789),
        ],
    )?;

    // Feature creation triggering an error due to a wrong field name:
    match layer.create_feature_fields(
        Geometry::from_wkt("POINT (46.50 22.50)")?,
        &["Abcd", "Value"],
        &[
            FieldValue::StringValue("Feature 2".to_string()),
            FieldValue::RealValue(0.789),
        ],
    ) {
        Ok(v) => v,
        Err(err) => println!("{err}"),
    };

    Ok(())
}

fn main() {
    example_1().unwrap();
    example_2().unwrap();
}
