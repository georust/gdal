use std::{ops::Add, path::Path};

#[cfg(feature = "datetime")]
use chrono::Duration;
use gdal::{
    errors::Error, Dataset, DatasetCommon, Defn, Driver, DriverCommon, Feature, FieldDefn,
    FieldValue, VectorDatasetCommon, VectorLayerCommon,
};

#[cfg(feature = "datetime")]
fn run() -> Result<(), Error> {
    println!("gdal crate was build with datetime support");

    let mut dataset_a = Dataset::open(Path::new("fixtures/points_with_datetime.json"))?;
    let layer_a = dataset_a.layer(0)?;

    // Create a new dataset:
    let _ = std::fs::remove_file("/tmp/later.geojson");
    let drv = Driver::get("GeoJSON")?;
    let mut ds = drv.create_vector_only(Path::new("/tmp/later.geojson"))?;
    let lyr = ds.create_layer()?;

    // Copy the origin layer shema to the destination layer:
    for field in layer_a.defn().fields() {
        let field_defn = FieldDefn::new(&field.name(), field.field_type())?;
        field_defn.set_width(field.width());
        field_defn.add_to_layer(&lyr)?;
    }

    // Get the definition to use on each feature:
    let defn = Defn::from_layer(&lyr);

    for feature_a in layer_a.features() {
        let mut ft = Feature::new(&defn)?;
        ft.set_geometry(feature_a.geometry().clone())?;
        // copy each field value of the feature:
        for field in defn.fields() {
            ft.set_field(
                &field.name(),
                &match feature_a.field(&field.name())? {
                    // add one day to dates
                    FieldValue::DateValue(value) => {
                        println!("{} = {}", field.name(), value);
                        FieldValue::DateValue(value.add(Duration::days(1)))
                    }

                    // add 6 hours to datetimes
                    FieldValue::DateTimeValue(value) => {
                        println!("{} = {}", field.name(), value);
                        FieldValue::DateTimeValue(value.add(Duration::hours(6)))
                    }
                    v => v,
                },
            )?;
        }
        // Add the feature to the layer:
        ft.create(&lyr)?;
    }
    Ok(())
}

#[cfg(not(feature = "datetime"))]
fn run() -> Result<(), Error> {
    println!("gdal crate was build without datetime support");
    Ok(())
}

fn main() {
    run().unwrap();
}
