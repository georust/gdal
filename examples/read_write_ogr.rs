use gdal::errors::Result;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::Dataset;
use gdal::{vector::*, DriverManager};
use std::fs;
use std::path::Path;

fn run() -> Result<()> {
    let dataset_a = Dataset::open(Path::new("fixtures/roads.geojson"))?;
    let mut layer_a = dataset_a.layer(0)?;
    let fields_defn = layer_a
        .defn()
        .fields()
        .map(|field| (field.name(), field.field_type(), field.width()))
        .collect::<Vec<_>>();

    // Create a new dataset:
    let path = std::env::temp_dir().join("abcde.shp");
    let _ = fs::remove_file(&path);
    let drv = DriverManager::get_driver_by_name("ESRI Shapefile")?;
    let mut ds = drv.create_vector_only(path.to_str().unwrap())?;
    let lyr = ds.create_layer(Default::default())?;

    // Copy the origin layer shema to the destination layer:
    for fd in &fields_defn {
        let field_defn = FieldDefn::new(&fd.0, fd.1)?;
        field_defn.set_width(fd.2);
        field_defn.add_to_layer(&lyr)?;
    }

    // Prepare the origin and destination spatial references objects:
    let spatial_ref_src = SpatialRef::from_epsg(4326)?;
    let spatial_ref_dst = SpatialRef::from_epsg(3025)?;

    // And the feature used to actually transform the geometries:
    let htransform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst)?;

    // Get the definition to use on each feature:
    let defn = Defn::from_layer(&lyr);

    for feature_a in layer_a.features() {
        // Create the new feature
        let mut ft = Feature::new(&defn)?;

        // Get the original geometry
        if let Some(geom) = feature_a.geometry() {
            // Get a new transformed geometry:
            let new_geom = geom.transform(&htransform)?;

            // Set the new feature's geometry
            ft.set_geometry(new_geom)?;
        }

        // copy each field value of the feature:
        for fd in &fields_defn {
            if let Some(value) = feature_a.field(&fd.0)? {
                ft.set_field(&fd.0, &value)?;
            }
        }
        // Add the feature to the layer:
        ft.create(&lyr)?;
    }

    Ok(())
}

fn main() {
    run().unwrap();
}
