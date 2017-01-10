extern crate gdal;
use std::path::Path;
use std::fs;
use gdal::vector::{Defn, Driver, Feature, FieldDefn, Geometry, OFT_REAL, OFT_STRING};

fn main(){
    fs::remove_file("/tmp/abcde.geojson");
    let drv = Driver::get("GeoJSON").unwrap();
    let mut ds = drv.create(Path::new("/tmp/abcde.geojson")).unwrap();

    let lyr = ds.create_layer().unwrap();

    let field_defn = FieldDefn::new("Name", OFT_STRING);
    field_defn.set_width(80);
    field_defn.add_to_layer(&lyr);

    let field_defn = FieldDefn::new("Value", OFT_REAL);
    field_defn.add_to_layer(&lyr);

    let defn = Defn::new_from_layer(&lyr);

    let mut ft = Feature::new(&defn);
    ft.set_geometry(Geometry::from_wkt("POINT (45.21 21.76)").unwrap());
    ft.set_field_string("Name", "Feature 1");
    ft.set_field_double("Value", 45.78);
    ft.create(&lyr);

    let mut ft = Feature::new(&defn);
    ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)").unwrap());
    ft.set_field_string("Name", "Feature 2");
    ft.set_field_double("Value", 0.789);
    ft.create(&lyr);
}
