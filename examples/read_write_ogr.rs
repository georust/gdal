extern crate gdal;

use std::fs;
use std::path::Path;
use std::fmt::Display;
use gdal::vector::{Dataset, Defn, Driver, Feature, FieldDefn, Geometry, OFT_STRING, OFT_REAL, OFT_INTEGER};
use gdal::spatial_ref::{SpatialRef, CoordTransform};

fn main() {
    // let mut dataset_a = Dataset::open(Path::new("/home/mz/Bureau/nuts2_data.geojson")).unwrap();
    // let layer_a = dataset_a.layer(0).unwrap();
    let mut dataset_a = Dataset::open(Path::new("/home/mz/code_rust/rust-geos/examples/GrandParisMunicipalities.geojson")).unwrap();
    let layer_a = dataset_a.layer(0).unwrap();
    let fields_defn = layer_a.defn().fields()
            .map(|field| (field.name(), field.get_type(), field.get_width()))
            .collect::<Vec<_>>();
    // println!("{:?}", fields_defn);

    // Create a new dataset :
    fs::remove_file("/tmp/abcde.shp");
    let drv = Driver::get("ESRI Shapefile").unwrap();
    let mut ds = drv.create(Path::new("/tmp/abcde.shp")).unwrap();
    let lyr = ds.create_layer().unwrap();

    // Copy the origin layer shema to the destination layer :
    for fd in &fields_defn {
        println!("{:?} {:?} {:?}", fd.0, fd.1, fd.2);
        let field_defn = FieldDefn::new(&fd.0, fd.1);
        field_defn.set_width(fd.2);
        field_defn.add_to_layer(&lyr);
    }

    // Prepare the origin and destination spatial references objects :
    let spatial_ref_src = SpatialRef::from_epsg(4326).unwrap();
    let spatial_ref_dst = SpatialRef::from_epsg(3025).unwrap();

    // And the feature used to actually transform the geometries :
    let htransform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst).unwrap();

    // Get the definition to use on each feature :
    let defn = Defn::new_from_layer(&lyr);

    for (i, feature_a) in layer_a.features().enumerate() {
        // Get the original geometry :
        let geom = feature_a.geometry();
        // Get a new transformed geometry :
        let new_geom = unsafe { geom.transform_new(&htransform).unwrap() };
        // Create the new feature, set its geometry :
        let mut ft = Feature::new(&defn);
        ft.set_geometry(new_geom);
        // copy each field value of the feature :
        for fd in &fields_defn {
            if fd.1 == 2 {
                let val = feature_a.field(&fd.0).unwrap().as_real();
                ft.set_field_double(&fd.0, val);
            } else if fd.1 == 4 {
                let val = &feature_a.field(&fd.0).unwrap().as_string();
                ft.set_field_string(&fd.0, val);
            } else if fd.1 == 0 {
                let val = &feature_a.field(&fd.0).unwrap().as_int();
                ft.set_field_integer(&fd.0, *val);
            }
        }
        // Add the feature to the layer :
        ft.create(&lyr);
    }
}
