extern crate gdal;

use std::fs;
use std::path::Path;
use gdal::vector::*;
use gdal::spatial_ref::{SpatialRef, CoordTransform};

fn main() {
    let mut dataset_a = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
    let layer_a = dataset_a.layer(0).unwrap();
    let fields_defn = layer_a.defn().fields()
            .map(|field| (field.name(), field.field_type(), field.width()))
            .collect::<Vec<_>>();

    // Create a new dataset :
    fs::remove_file("/tmp/abcde.shp");
    let drv = Driver::get("ESRI Shapefile").unwrap();
    let mut ds = drv.create(Path::new("/tmp/abcde.shp")).unwrap();
    let lyr = ds.create_layer().unwrap();

    // Copy the origin layer shema to the destination layer :
    for fd in &fields_defn {
        let field_defn = FieldDefn::new(&fd.0, fd.1).unwrap();
        field_defn.set_width(fd.2);
        field_defn.add_to_layer(&lyr);
    }

    // Prepare the origin and destination spatial references objects :
    let spatial_ref_src = SpatialRef::from_epsg(4326).unwrap();
    let spatial_ref_dst = SpatialRef::from_epsg(3025).unwrap();

    // And the feature used to actually transform the geometries :
    let htransform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst).unwrap();

    // Get the definition to use on each feature :
    let defn = Defn::from_layer(&lyr);

    for feature_a in layer_a.features() {
        // Get the original geometry :
        let geom = feature_a.geometry();
        // Get a new transformed geometry :
        let new_geom = geom.transform(&htransform).unwrap();
        // Create the new feature, set its geometry :
        let mut ft = Feature::new(&defn).unwrap();
        ft.set_geometry(new_geom);
        // copy each field value of the feature :
        for fd in &fields_defn {
            ft.set_field(&fd.0, &feature_a.field(&fd.0).unwrap()).unwrap();
        }
        // Add the feature to the layer :
        ft.create(&lyr);
    }
}
