extern crate gdal;

use std::fs;
use std::path::Path;
use std::fmt::Display;
use gdal::vector::{Dataset, Defn, Driver, Feature, FieldDefn, Geometry, OFT_REAL, OFT_STRING};
use gdal::spatial_ref::{SpatialRef, CoordTransform};

fn main() {
    // let mut dataset_a = Dataset::open(Path::new("/home/mz/Bureau/nuts2_data.geojson")).unwrap();
    // let layer_a = dataset_a.layer(0).unwrap();
    let mut dataset_a = Dataset::open(Path::new("/home/mz/code_rust/rust-geos/examples/GrandParisMunicipalities.geojson")).unwrap();
    let layer_a = dataset_a.layer(0).unwrap();

    fs::remove_file("/tmp/abcde.shp");
    let drv = Driver::get("ESRI Shapefile").unwrap();
    let mut ds = drv.create(Path::new("/tmp/abcde.shp")).unwrap();
    let lyr = ds.create_layer().unwrap();

    let field_defn = FieldDefn::new("Name", OFT_STRING);
    field_defn.set_width(80);
    field_defn.add_to_layer(&lyr);

    let defn = Defn::new_from_layer(&lyr);
    let spatial_ref_src = SpatialRef::from_epsg(4326).unwrap();
    let spatial_ref_dst = SpatialRef::from_epsg(3025).unwrap();
    let htransform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst).unwrap();

    for (i, feature_a) in layer_a.features().enumerate() {
        let geom = feature_a.geometry();
        let new_geom = unsafe { geom.transform_new(&htransform).unwrap() };
        let mut ft = Feature::new(&defn);
        ft.set_geometry(new_geom);
        ft.set_field_string("Name", "Feature");
        ft.create(&lyr);
    }
}
