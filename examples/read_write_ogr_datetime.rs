use gdal::{vector::LayerAccess, DriverManager};

fn run() -> gdal::errors::Result<()> {
    use chrono::Duration;
    use gdal::vector::{Defn, Feature, FieldDefn, FieldValue};
    use gdal::Dataset;
    use std::ops::Add;
    use std::path::Path;

    println!("gdal crate was build with datetime support");

    let dataset_a = Dataset::open(Path::new("fixtures/points_with_datetime.json"))?;
    let mut layer_a = dataset_a.layer(0)?;

    // Create a new dataset:
    let path = std::env::temp_dir().join("later.geojson");
    let _ = std::fs::remove_file(&path);
    let drv = DriverManager::get_driver_by_name("GeoJSON")?;
    let mut ds = drv.create_vector_only(path.to_str().unwrap())?;
    let lyr = ds.create_layer(Default::default())?;

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
        if let Some(geom) = feature_a.geometry() {
            ft.set_geometry(geom.clone())?;
        }
        // copy each field value of the feature:
        for field in defn.fields() {
            ft.set_field(
                &field.name(),
                &match feature_a.field(field.name())?.unwrap() {
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

fn main() {
    run().unwrap();
}
