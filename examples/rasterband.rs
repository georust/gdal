extern crate gdal;

use std::path::Path;
use gdal::raster::{Dataset, RasterBand};
use gdal::metadata::Metadata;

fn main() {

    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.get_description());

    let rasterband: RasterBand = dataset.get_rasterband(1).unwrap();
    println!("rasterband description: {:?}", rasterband.get_description());
    println!("rasterband no_data_value: {:?}", rasterband.get_no_data_value());
    println!("rasterband type: {:?}", rasterband.get_band_type());
    println!("rasterband scale: {:?}", rasterband.get_scale());
    println!("rasterband offset: {:?}", rasterband.get_offset());
    let rv = rasterband.read_as::<u8>(
        (20, 30),
        (2, 3),
        (2, 3)
    );
    println!("{:?}", rv.data);
}
