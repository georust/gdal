extern crate gdal;
use std::path::Path;
use std::fs;
use gdal::vector::{Defn, Driver, Feature, FieldDefn, Geometry, OFT_INTEGER, OFT_REAL, OFT_STRING, FieldValue};

fn main(){
    {
        fs::remove_file("/tmp/abcde.geojson");
        let drv = Driver::get("GeoJSON").unwrap();
        let mut ds = drv.create(Path::new("/tmp/output1.geojson")).unwrap();

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

    {
        let driver = Driver::get("GeoJSON").unwrap();
        let mut ds = driver.create(Path::new("/tmp/output2.geojson")).unwrap();
        let mut layer = ds.create_layer().unwrap();

        layer.create_defn_fields(&[FieldDefn::new("Name", OFT_STRING), FieldDefn::new("Value", OFT_REAL)]);
        // Shortcut for :
        // let field_defn = FieldDefn::new("Name", OFT_STRING);
        // field_defn.add_to_layer(&layer);
        // let field_defn = FieldDefn::new("Value", OFT_REAL);
        // field_defn.add_to_layer(&layer);

        let ft = layer.create_feature_fields(
            Geometry::from_wkt("POINT (45.21 21.76)").unwrap(),
            &["Name", "Value"],
            &[FieldValue::StringValue("Feature 1".to_string()), FieldValue::RealValue(45.78)]
            );
        // Shortcut for :
        // let defn = Defn::new_from_layer(&layer);
        // let mut ft = Feature::new(&defn);
        // ft.set_geometry(Geometry::from_wkt("POINT (1 2)").unwrap());
        // ft.set_field("Name", OFT_STRING, "Feature 1");
        // ft.set_field("Value", OFT_REAL, 30);
        // ft.create(&lyr);
        let ft = layer.create_feature_fields(
            Geometry::from_wkt("POINT (46.50 22.50)").unwrap(),
            &["Name", "Value"],
            &[FieldValue::StringValue("Feature 2".to_string()), FieldValue::RealValue(0.789)]
            );
    }

}
