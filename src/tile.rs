extern crate sync;

use std::os::args;
use std::path::Path;

#[allow(dead_code)]
mod gdal;


fn main() {
    let memory_driver = gdal::get_driver("MEM").unwrap();

    let source = gdal::open(&Path::new(args()[1])).unwrap();
    println!(
        "size: {}, bands: {}",
        source.get_raster_size(),
        source.get_raster_count()
    );

    let tile = memory_driver.create("", 256, 256, 3).unwrap();
    println!("tile size: {}", tile.get_raster_size());
}
