use gdal::errors::Result;
use gdal::vector::{Defn, Feature, FieldDefn, Geometry, OGRFieldType};
use gdal::DriverManager;
use std::fs;

fn main() -> Result<()> {
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

    let name_idx = defn.field_index("Name")?;
    let value_idx = defn.field_index("Value")?;

    // 1st feature:
    let mut ft = Feature::new(&defn)?;
    ft.set_geometry(Geometry::from_wkt("POINT (45.21 21.76)")?)?;
    ft.set_field_string(name_idx, "Feature 1")?;
    ft.set_field_double(value_idx, 45.78)?;
    ft.create(&lyr)?;

    // 2nd feature:
    let mut ft = Feature::new(&defn)?;
    ft.set_field_double(value_idx, 0.789)?;
    ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)")?)?;
    ft.set_field_string(name_idx, "Feature 2")?;
    ft.create(&lyr)?;

    // Feature triggering an error due to a wrong field name:
    let mut ft = Feature::new(&defn)?;
    ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)")?)?;
    ft.set_field_string(name_idx, "Feature 2")?;
    match ft.set_field_double(value_idx, 0.789) {
        Ok(v) => v,
        Err(err) => println!("{err}"),
    };
    ft.create(&lyr)?;

    Ok(())
}
